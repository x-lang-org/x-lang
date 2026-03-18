use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Manifest {
    pub package: Option<PackageConfig>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub dependencies: BTreeMap<String, Dependency>,
    #[serde(
        default,
        rename = "dev-dependencies",
        skip_serializing_if = "BTreeMap::is_empty"
    )]
    pub dev_dependencies: BTreeMap<String, Dependency>,
    #[serde(
        default,
        rename = "build-dependencies",
        skip_serializing_if = "BTreeMap::is_empty"
    )]
    pub build_dependencies: BTreeMap<String, Dependency>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace: Option<WorkspaceConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lib: Option<LibTarget>,
    #[serde(default, rename = "bin", skip_serializing_if = "Vec::is_empty")]
    pub bins: Vec<BinTarget>,
    #[serde(default, rename = "test", skip_serializing_if = "Vec::is_empty")]
    pub tests: Vec<TestTarget>,
    #[serde(default, rename = "bench", skip_serializing_if = "Vec::is_empty")]
    pub benches: Vec<BenchTarget>,
    #[serde(default, rename = "example", skip_serializing_if = "Vec::is_empty")]
    pub examples: Vec<ExampleTarget>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub profile: BTreeMap<String, Profile>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub features: BTreeMap<String, Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageConfig {
    pub name: String,
    #[serde(default = "default_version")]
    pub version: String,
    #[serde(default = "default_edition")]
    pub edition: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authors: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub readme: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub homepage: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keywords: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub categories: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publish: Option<bool>,
    #[serde(rename = "default-run", skip_serializing_if = "Option::is_none")]
    pub default_run: Option<String>,
    #[serde(default = "default_true", skip_serializing_if = "is_true")]
    pub autobins: bool,
    #[serde(default = "default_true", skip_serializing_if = "is_true")]
    pub autoexamples: bool,
    #[serde(default = "default_true", skip_serializing_if = "is_true")]
    pub autotests: bool,
    #[serde(default = "default_true", skip_serializing_if = "is_true")]
    pub autobenches: bool,
}

fn default_version() -> String {
    "0.1.0".to_string()
}
fn default_edition() -> String {
    "2025".to_string()
}
fn default_true() -> bool {
    true
}
fn is_true(v: &bool) -> bool {
    *v
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Dependency {
    Simple(String),
    Detailed(DetailedDependency),
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DetailedDependency {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub git: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rev: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub features: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub optional: Option<bool>,
    #[serde(rename = "default-features", skip_serializing_if = "Option::is_none")]
    pub default_features: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub registry: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WorkspaceConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub members: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclude: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dependencies: Option<BTreeMap<String, Dependency>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibTarget {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinTarget {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestTarget {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchTarget {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExampleTarget {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Profile {
    #[serde(rename = "opt-level", skip_serializing_if = "Option::is_none")]
    pub opt_level: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub debug: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lto: Option<bool>,
    #[serde(rename = "codegen-units", skip_serializing_if = "Option::is_none")]
    pub codegen_units: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub incremental: Option<bool>,
    #[serde(rename = "overflow-checks", skip_serializing_if = "Option::is_none")]
    pub overflow_checks: Option<bool>,
}

impl Manifest {
    pub fn from_path(path: &Path) -> Result<Self, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("无法读取 {}: {}", path.display(), e))?;
        toml::from_str(&content).map_err(|e| format!("解析 {} 失败: {}", path.display(), e))
    }

    pub fn to_string_pretty(&self) -> Result<String, String> {
        toml::to_string_pretty(self).map_err(|e| format!("序列化清单失败: {}", e))
    }

    pub fn find_manifest_path(start: &Path) -> Option<PathBuf> {
        let mut current = if start.is_file() {
            start.parent()?.to_path_buf()
        } else {
            start.to_path_buf()
        };
        loop {
            let manifest = current.join("x.toml");
            if manifest.exists() {
                return Some(manifest);
            }
            if !current.pop() {
                return None;
            }
        }
    }

    pub fn package_name(&self) -> Option<&str> {
        self.package.as_ref().map(|p| p.name.as_str())
    }

    pub fn package_version(&self) -> Option<&str> {
        self.package.as_ref().map(|p| p.version.as_str())
    }

    pub fn default_bin(name: &str) -> Self {
        Manifest {
            package: Some(PackageConfig {
                name: name.to_string(),
                version: "0.1.0".to_string(),
                edition: "2025".to_string(),
                authors: None,
                description: None,
                license: None,
                readme: None,
                homepage: None,
                repository: None,
                keywords: None,
                categories: None,
                publish: None,
                default_run: None,
                autobins: true,
                autoexamples: true,
                autotests: true,
                autobenches: true,
            }),
            ..Default::default()
        }
    }

    pub fn default_lib(name: &str) -> Self {
        let mut m = Self::default_bin(name);
        m.lib = Some(LibTarget {
            name: Some(name.to_string()),
            path: Some("src/lib.x".to_string()),
        });
        m
    }
}

impl Dependency {
    pub fn version(&self) -> Option<&str> {
        match self {
            Dependency::Simple(v) => Some(v.as_str()),
            Dependency::Detailed(d) => d.version.as_deref(),
        }
    }

    pub fn is_path(&self) -> bool {
        matches!(self, Dependency::Detailed(d) if d.path.is_some())
    }

    pub fn is_git(&self) -> bool {
        matches!(self, Dependency::Detailed(d) if d.git.is_some())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_roundtrip_default_bin() {
        let m = Manifest::default_bin("demo");
        let s = m.to_string_pretty().expect("serialize ok");
        let parsed: Manifest = toml::from_str(&s).expect("parse ok");
        assert_eq!(parsed.package_name(), Some("demo"));
        assert_eq!(parsed.package_version(), Some("0.1.0"));
    }

    #[test]
    fn find_manifest_path_walks_upwards() {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path().join("a").join("b").join("c");
        std::fs::create_dir_all(&root).expect("mkdirs");
        let manifest_path = dir.path().join("a").join("x.toml");
        std::fs::write(&manifest_path, "package = { name = \"p\" }\n").expect("write");

        let found = Manifest::find_manifest_path(&root).expect("should find");
        assert_eq!(found, manifest_path);
    }
}
