use crate::config::{Credentials, GlobalConfig};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::{collections::hash_map::DefaultHasher, hash::{Hash, Hasher}, io::Read};
use semver::Version;

pub const DEFAULT_REGISTRY: &str = "https://registry.x-lang.org";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageInfo {
    pub name: String,
    pub description: Option<String>,
    pub max_version: String,
    pub versions: Vec<VersionInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionInfo {
    pub version: String,
    pub yanked: bool,
    pub checksum: String,
    pub dependencies: Vec<DepInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepInfo {
    pub name: String,
    pub version_req: String,
    pub optional: bool,
}

pub struct RegistryClient {
    pub url: String,
    pub token: Option<String>,
}

impl RegistryClient {
    pub fn new(registry_name: Option<&str>) -> Self {
        let config = GlobalConfig::load();
        let creds = Credentials::load();
        let registry_config = config.registry.as_ref();
        let selected_registry = registry_name
            .map(str::to_string)
            .or_else(|| registry_config.and_then(|r| r.default.clone()));

        let url = selected_registry
            .as_deref()
            .and_then(|name| registry_config.and_then(|r| r.registries.get(name)))
            .map(|e| e.index.clone())
            .unwrap_or_else(|| DEFAULT_REGISTRY.to_string());

        let token = creds.get_token(registry_name).map(|s| s.to_string());
        RegistryClient { url, token }
    }

    pub fn from_registry_name(registry_name: Option<&str>) -> Result<Self, String> {
        let config = GlobalConfig::load_checked()?;
        let creds = Credentials::load();
        let registry_config = config.registry.as_ref();

        let selected_name = match registry_name {
            Some(name) => Some(name.to_string()),
            None => Some(
                registry_config
                    .and_then(|r| r.default.clone())
                    .ok_or_else(missing_default_registry_error)?,
            ),
        };

        let url = selected_name
            .as_deref()
            .and_then(|name| registry_config.and_then(|r| r.registries.get(name)))
            .map(|entry| entry.index.clone())
            .ok_or_else(|| missing_registry_error(selected_name.as_deref().unwrap_or("default")))?;

        let token_registry_name = registry_name.or(selected_name.as_deref());
        let token = creds.get_token(token_registry_name).map(|s| s.to_string());

        Ok(RegistryClient { url, token })
    }

    pub fn search(&self, query: &str, limit: usize) -> Result<Vec<PackageInfo>, String> {
        let index_dir = self.local_index_dir().ok_or_else(|| {
            format!(
                "注册表搜索尚未实现（注册表: {}，查询: {}，限制: {}）\n\
                 当前仅支持本地文件注册表索引目录搜索",
                self.url, query, limit
            )
        })?;

        let mut results = Vec::new();
        let needle = query.to_lowercase();

        if !index_dir.exists() {
            return Ok(results);
        }

        let entries = std::fs::read_dir(&index_dir)
            .map_err(|e| format!("无法读取本地注册表索引 {}: {}", index_dir.display(), e))?;

        for entry in entries {
            let entry = entry.map_err(|e| format!("无法读取注册表索引项: {}", e))?;
            let path = entry.path();
            if path.extension().is_none_or(|ext| ext != "json") {
                continue;
            }

            let content = std::fs::read_to_string(&path)
                .map_err(|e| format!("无法读取注册表元数据 {}: {}", path.display(), e))?;
            let pkg: PackageInfo = serde_json::from_str(&content)
                .map_err(|e| format!("无法解析注册表元数据 {}: {}", path.display(), e))?;

            let description = pkg.description.as_deref().unwrap_or("").to_lowercase();
            if pkg.name.to_lowercase().contains(&needle) || description.contains(&needle) {
                results.push(pkg);
            }
        }

        results.sort_by(|a, b| a.name.cmp(&b.name).then(a.max_version.cmp(&b.max_version)));
        if limit < results.len() {
            results.truncate(limit);
        }

        Ok(results)
    }

    pub fn get_package(&self, name: &str) -> Result<PackageInfo, String> {
        let index_dir = self.local_index_dir().ok_or_else(|| {
            format!("注册表查询尚未实现（注册表: {}，包: {}）", self.url, name)
        })?;

        let path = index_dir.join(format!("{}.json", name));
        let content = std::fs::read_to_string(&path)
            .map_err(|e| format!("无法读取注册表元数据 {}: {}", path.display(), e))?;
        serde_json::from_str(&content)
            .map_err(|e| format!("无法解析注册表元数据 {}: {}", path.display(), e))
    }

    pub fn publish(&self, tarball: &[u8]) -> Result<(), String> {
        let registry_root = self.local_registry_root().ok_or_else(|| {
            format!(
                "发布功能尚未实现（注册表: {}）\n\
                 当前仅支持本地文件注册表发布",
                self.url
            )
        })?;

        let package_info = self.build_package_info_from_tarball(tarball)?;
        let version = package_info.max_version.clone();
        let tarball_name = format!("{}-{}.tar.gz", package_info.name, version);
        let crate_dir = registry_root.join("crate").join(&package_info.name).join(&version);
        let tarball_path = crate_dir.join(&tarball_name);
        if tarball_path.exists() {
            return Err(format!("{} {} 已存在于本地注册表中", package_info.name, version));
        }

        std::fs::create_dir_all(&crate_dir)
            .map_err(|e| format!("无法创建本地注册表目录 {}: {}", crate_dir.display(), e))?;
        std::fs::write(&tarball_path, tarball)
            .map_err(|e| format!("无法写入本地注册表包 {}: {}", tarball_path.display(), e))?;

        let index_dir = registry_root.join("index");
        std::fs::create_dir_all(&index_dir)
            .map_err(|e| format!("无法创建注册表索引目录 {}: {}", index_dir.display(), e))?;
        let index_path = index_dir.join(format!("{}.json", package_info.name));

        let mut merged = if index_path.exists() {
            let content = std::fs::read_to_string(&index_path)
                .map_err(|e| format!("无法读取注册表元数据 {}: {}", index_path.display(), e))?;
            serde_json::from_str::<PackageInfo>(&content)
                .map_err(|e| format!("无法解析注册表元数据 {}: {}", index_path.display(), e))?
        } else {
            PackageInfo {
                name: package_info.name.clone(),
                description: package_info.description.clone(),
                max_version: package_info.max_version.clone(),
                versions: Vec::new(),
            }
        };

        if merged.versions.iter().any(|existing| existing.version == version) {
            return Err(format!("{} {} 已存在于本地注册表索引中", merged.name, version));
        }

        if merged.description.is_none() {
            merged.description = package_info.description.clone();
        }
        merged.versions.extend(package_info.versions);
        merged.max_version = select_highest_version(&merged.versions)?;

        let json = serde_json::to_string_pretty(&merged)
            .map_err(|e| format!("无法序列化注册表元数据: {}", e))?;
        std::fs::write(&index_path, json)
            .map_err(|e| format!("无法写入注册表元数据 {}: {}", index_path.display(), e))
    }

    pub fn yank(&self, name: &str, version: &str) -> Result<(), String> {
        if self.token.is_none() {
            return Err("未登录，请先运行 `x login`".to_string());
        }

        let index_path = self.local_package_index_path(name).ok_or_else(|| {
            format!(
                "撤回功能尚未实现（注册表: {}，包: {}@{}）\n当前仅支持本地文件注册表撤回",
                self.url, name, version
            )
        })?;

        self.update_yank_state(&index_path, name, version, true)
    }

    pub fn unyank(&self, name: &str, version: &str) -> Result<(), String> {
        if self.token.is_none() {
            return Err("未登录，请先运行 `x login`".to_string());
        }

        let index_path = self.local_package_index_path(name).ok_or_else(|| {
            format!(
                "取消撤回功能尚未实现（注册表: {}，包: {}@{}）\n当前仅支持本地文件注册表取消撤回",
                self.url, name, version
            )
        })?;

        self.update_yank_state(&index_path, name, version, false)
    }

    pub fn add_owner(&self, name: &str, owner: &str) -> Result<(), String> {
        if self.token.is_none() {
            return Err("未登录，请先运行 `x login`".to_string());
        }
        Err(format!(
            "添加所有者功能尚未实现（注册表: {}，包: {}，所有者: {}）",
            self.url, name, owner
        ))
    }

    pub fn remove_owner(&self, name: &str, owner: &str) -> Result<(), String> {
        if self.token.is_none() {
            return Err("未登录，请先运行 `x login`".to_string());
        }
        Err(format!(
            "移除所有者功能尚未实现（注册表: {}，包: {}，所有者: {}）",
            self.url, name, owner
        ))
    }

    pub fn list_owners(&self, name: &str) -> Result<Vec<String>, String> {
        Err(format!(
            "列出所有者功能尚未实现（注册表: {}，包: {}）",
            self.url, name
        ))
    }

    pub fn read_package_tarball(&self, name: &str, version: &str) -> Result<Vec<u8>, String> {
        let registry_root = self.local_registry_root().ok_or_else(|| {
            format!(
                "从注册表安装尚未实现（注册表: {}，包: {}@{}）\n\
                 当前仅支持本地文件注册表安装",
                self.url, name, version
            )
        })?;

        let tarball_path = registry_root
            .join("crate")
            .join(name)
            .join(version)
            .join(format!("{}-{}.tar.gz", name, version));
        std::fs::read(&tarball_path)
            .map_err(|e| format!("无法读取本地注册表包 {}: {}", tarball_path.display(), e))
    }

    fn local_index_dir(&self) -> Option<PathBuf> {
        let root = self.local_registry_root()?;
        let index_dir = root.join("index");
        if index_dir.exists() {
            Some(index_dir)
        } else {
            Some(root)
        }
    }

    fn local_registry_root(&self) -> Option<PathBuf> {
        if self.url.starts_with("http://") || self.url.starts_with("https://") {
            None
        } else {
            Some(Path::new(&self.url).to_path_buf())
        }
    }

    fn local_package_index_path(&self, name: &str) -> Option<PathBuf> {
        self.local_index_dir()
            .map(|index_dir| index_dir.join(format!("{}.json", name)))
    }

    fn update_yank_state(
        &self,
        index_path: &Path,
        name: &str,
        version: &str,
        yanked: bool,
    ) -> Result<(), String> {
        let content = std::fs::read_to_string(index_path)
            .map_err(|e| format!("无法读取注册表元数据 {}: {}", index_path.display(), e))?;
        let mut pkg: PackageInfo = serde_json::from_str(&content)
            .map_err(|e| format!("无法解析注册表元数据 {}: {}", index_path.display(), e))?;

        let selected = pkg
            .versions
            .iter_mut()
            .find(|entry| entry.version == version)
            .ok_or_else(|| format!("本地注册表中未找到 {}@{}", name, version))?;
        selected.yanked = yanked;

        let updated = serde_json::to_string_pretty(&pkg)
            .map_err(|e| format!("无法序列化注册表元数据: {}", e))?;
        std::fs::write(index_path, updated)
            .map_err(|e| format!("无法写入注册表元数据 {}: {}", index_path.display(), e))
    }

    fn build_package_info_from_tarball(&self, tarball: &[u8]) -> Result<PackageInfo, String> {
        let cursor = std::io::Cursor::new(tarball);
        let gz = flate2::read::GzDecoder::new(cursor);
        let mut archive = tar::Archive::new(gz);
        let mut manifest = None;

        for entry in archive.entries().map_err(|e| format!("无法读取打包内容: {}", e))? {
            let mut entry = entry.map_err(|e| format!("无法读取打包条目: {}", e))?;
            let path = entry
                .path()
                .map_err(|e| format!("无法读取打包路径: {}", e))?
                .into_owned();

            if path.file_name().is_some_and(|name| name == "x.toml") {
                let mut content = String::new();
                entry
                    .read_to_string(&mut content)
                    .map_err(|e| format!("无法读取打包内的 x.toml: {}", e))?;
                manifest = Some(
                    toml::from_str::<crate::manifest::Manifest>(&content)
                        .map_err(|e| format!("无法解析打包内的 x.toml: {}", e))?,
                );
                break;
            }
        }

        let manifest = manifest.ok_or("打包内容中未找到 x.toml".to_string())?;
        let package = manifest.package.ok_or("打包内容缺少 [package] 元数据".to_string())?;
        let version = package.version.clone();
        let checksum = checksum_hex(tarball);

        Ok(PackageInfo {
            name: package.name,
            description: package.description,
            max_version: version.clone(),
            versions: vec![VersionInfo {
                version,
                yanked: false,
                checksum,
                dependencies: Vec::new(),
            }],
        })
    }
}

fn missing_registry_error(name: &str) -> String {
    format!(
        "配置中未定义注册表 `{}`（当前配置: {}）\n请设置 X_HOME 或在 config.toml 中添加 [registry.registries.{}]",
        name,
        crate::config::config_path().display(),
        name
    )
}

fn missing_default_registry_error() -> String {
    format!(
        "当前未配置默认注册表（当前配置: {}）\n请设置 X_HOME，并在 config.toml 中设置 [registry].default 和对应的 [registry.registries.<name>]",
        crate::config::config_path().display()
    )
}

fn select_highest_version(versions: &[VersionInfo]) -> Result<String, String> {
    versions
        .iter()
        .map(|info| {
            Version::parse(&info.version)
                .map(|parsed| (parsed, info.version.clone()))
                .map_err(|e| format!("无效的注册表版本 `{}`: {}", info.version, e))
        })
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .max_by(|a, b| a.0.cmp(&b.0))
        .map(|(_, original)| original)
        .ok_or("注册表包没有可用版本".to_string())
}

fn checksum_hex(bytes: &[u8]) -> String {
    let mut hasher = DefaultHasher::new();
    bytes.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}
