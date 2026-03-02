use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GlobalConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub registry: Option<RegistryConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub build: Option<BuildConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub install: Option<InstallConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub net: Option<NetConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RegistryConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub registries: BTreeMap<String, RegistryEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryEntry {
    pub index: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BuildConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jobs: Option<u32>,
    #[serde(rename = "target-dir", skip_serializing_if = "Option::is_none")]
    pub target_dir: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub incremental: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InstallConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub root: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NetConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry: Option<u32>,
    #[serde(rename = "git-fetch-with-cli", skip_serializing_if = "Option::is_none")]
    pub git_fetch_with_cli: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offline: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Credentials {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub registry: Option<CredentialRegistry>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub registries: BTreeMap<String, CredentialEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialRegistry {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialEntry {
    pub token: String,
}

pub fn x_home() -> PathBuf {
    if let Ok(home) = std::env::var("X_HOME") {
        return PathBuf::from(home);
    }
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".x")
}

pub fn config_path() -> PathBuf {
    x_home().join("config.toml")
}

pub fn credentials_path() -> PathBuf {
    x_home().join("credentials.toml")
}

pub fn install_root() -> PathBuf {
    x_home().join("bin")
}

pub fn _registry_cache() -> PathBuf {
    x_home().join("registry")
}

impl GlobalConfig {
    pub fn load() -> Self {
        let path = config_path();
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(config) = toml::from_str(&content) {
                    return config;
                }
            }
        }
        Self::default()
    }

    pub fn save(&self) -> Result<(), String> {
        let path = config_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("无法创建配置目录: {}", e))?;
        }
        let content =
            toml::to_string_pretty(self).map_err(|e| format!("序列化配置失败: {}", e))?;
        std::fs::write(&path, content).map_err(|e| format!("无法写入配置: {}", e))
    }
}

impl Credentials {
    pub fn load() -> Self {
        let path = credentials_path();
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(creds) = toml::from_str(&content) {
                    return creds;
                }
            }
        }
        Self::default()
    }

    pub fn save(&self) -> Result<(), String> {
        let path = credentials_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("无法创建凭证目录: {}", e))?;
        }
        let content =
            toml::to_string_pretty(self).map_err(|e| format!("序列化凭证失败: {}", e))?;
        std::fs::write(&path, content).map_err(|e| format!("无法写入凭证: {}", e))
    }

    pub fn get_token(&self, registry: Option<&str>) -> Option<&str> {
        match registry {
            None | Some("default") => self.registry.as_ref()?.token.as_deref(),
            Some(name) => self.registries.get(name).map(|e| e.token.as_str()),
        }
    }

    pub fn set_token(&mut self, registry: Option<&str>, token: String) {
        match registry {
            None | Some("default") => {
                self.registry = Some(CredentialRegistry {
                    token: Some(token),
                });
            }
            Some(name) => {
                self.registries
                    .insert(name.to_string(), CredentialEntry { token });
            }
        }
    }

    pub fn remove_token(&mut self, registry: Option<&str>) {
        match registry {
            None | Some("default") => {
                self.registry = None;
            }
            Some(name) => {
                self.registries.remove(name);
            }
        }
    }
}
