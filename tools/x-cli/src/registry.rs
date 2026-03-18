use crate::config::{Credentials, GlobalConfig};
use serde::{Deserialize, Serialize};

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

        let url = if let Some(name) = registry_name {
            config
                .registry
                .as_ref()
                .and_then(|r| r.registries.get(name))
                .map(|e| e.index.clone())
                .unwrap_or_else(|| DEFAULT_REGISTRY.to_string())
        } else {
            DEFAULT_REGISTRY.to_string()
        };

        let token = creds.get_token(registry_name).map(|s| s.to_string());
        RegistryClient { url, token }
    }

    pub fn search(&self, query: &str, limit: usize) -> Result<Vec<PackageInfo>, String> {
        Err(format!(
            "注册表搜索尚未实现（注册表: {}，查询: {}，限制: {}）\n\
             注册表功能将在 X 语言包生态建立后启用",
            self.url, query, limit
        ))
    }

    pub fn get_package(&self, name: &str) -> Result<PackageInfo, String> {
        Err(format!(
            "注册表查询尚未实现（注册表: {}，包: {}）",
            self.url, name
        ))
    }

    pub fn publish(&self, _tarball: &[u8]) -> Result<(), String> {
        if self.token.is_none() {
            return Err("未登录，请先运行 `x login`".to_string());
        }
        Err(format!(
            "发布功能尚未实现（注册表: {}）\n\
             注册表功能将在 X 语言包生态建立后启用",
            self.url
        ))
    }

    pub fn yank(&self, name: &str, version: &str) -> Result<(), String> {
        if self.token.is_none() {
            return Err("未登录，请先运行 `x login`".to_string());
        }
        Err(format!(
            "撤回功能尚未实现（注册表: {}，包: {}@{}）",
            self.url, name, version
        ))
    }

    pub fn unyank(&self, name: &str, version: &str) -> Result<(), String> {
        if self.token.is_none() {
            return Err("未登录，请先运行 `x login`".to_string());
        }
        Err(format!(
            "取消撤回功能尚未实现（注册表: {}，包: {}@{}）",
            self.url, name, version
        ))
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
}
