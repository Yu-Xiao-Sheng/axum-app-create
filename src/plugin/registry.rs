// Plugin registry
//
// Manages the installed plugins index at ~/.axum-app-create/plugins.toml.

use crate::error::{CliError, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Plugin source type
/// 插件来源
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum PluginSource {
    /// 本地路径 / Local path
    Local { path: PathBuf },
    /// Git 仓库 / Git repository
    Git {
        url: String,
        #[serde(default)]
        rev: Option<String>,
    },
    /// crates.io crate
    Crate { name: String, version: String },
}

/// Installed plugin registry entry
/// 已安装插件的注册信息
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PluginEntry {
    /// 插件名称 / Plugin name
    pub name: String,
    /// 插件版本 / Plugin version
    pub version: String,
    /// 是否启用 / Enabled status
    pub enabled: bool,
    /// 插件来源 / Plugin source
    pub source: PluginSource,
    /// 本地安装路径 / Local installation path
    pub install_path: PathBuf,
    /// 安装时间 / Installation timestamp
    pub installed_at: String,
}

/// Plugin registry
/// 插件注册表
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct PluginRegistry {
    /// 已安装的插件列表 / Installed plugins
    #[serde(default)]
    pub plugins: Vec<PluginEntry>,
}

impl PluginRegistry {
    /// Get the registry file path
    fn registry_path() -> Result<PathBuf> {
        let home = dirs::home_dir().ok_or_else(|| {
            CliError::Plugin(
                "Cannot determine home directory / 无法确定主目录".to_string(),
            )
        })?;
        Ok(home.join(".axum-app-create").join("plugins.toml"))
    }

    /// Load registry from ~/.axum-app-create/plugins.toml
    /// 从 ~/.axum-app-create/plugins.toml 加载注册表
    pub fn load() -> Result<Self> {
        let path = Self::registry_path()?;
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = std::fs::read_to_string(&path).map_err(|e| {
            tracing::warn!(
                "Failed to read plugin registry, using empty registry / 读取插件注册表失败，使用空注册表: {}",
                e
            );
            e
        })?;
        toml::from_str(&content).map_err(|e| {
            tracing::warn!(
                "Plugin registry corrupted, using empty registry / 插件注册表损坏，使用空注册表: {}",
                e
            );
            CliError::Plugin(format!(
                "Failed to parse plugin registry / 解析插件注册表失败: {}",
                e
            ))
        })
    }

    /// Save registry to ~/.axum-app-create/plugins.toml
    /// 保存注册表到 ~/.axum-app-create/plugins.toml
    pub fn save(&self) -> Result<()> {
        let path = Self::registry_path()?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self).map_err(|e| {
            CliError::Plugin(format!(
                "Failed to serialize plugin registry / 序列化插件注册表失败: {}",
                e
            ))
        })?;
        std::fs::write(&path, content)?;
        Ok(())
    }

    /// Add a plugin entry
    /// 添加插件条目
    pub fn add(&mut self, entry: PluginEntry) {
        // Remove existing entry with same name first
        self.plugins.retain(|p| p.name != entry.name);
        self.plugins.push(entry);
    }

    /// Remove a plugin entry by name, returns the removed entry
    /// 移除插件条目
    pub fn remove(&mut self, name: &str) -> Option<PluginEntry> {
        if let Some(pos) = self.plugins.iter().position(|p| p.name == name) {
            Some(self.plugins.remove(pos))
        } else {
            None
        }
    }

    /// Find a plugin by name
    /// 查找插件
    pub fn find(&self, name: &str) -> Option<&PluginEntry> {
        self.plugins.iter().find(|p| p.name == name)
    }

    /// Find a plugin by name (mutable)
    /// 查找插件（可变引用）
    pub fn find_mut(&mut self, name: &str) -> Option<&mut PluginEntry> {
        self.plugins.iter_mut().find(|p| p.name == name)
    }

    /// Get all enabled plugins
    /// 获取所有启用的插件
    pub fn enabled_plugins(&self) -> Vec<&PluginEntry> {
        self.plugins.iter().filter(|p| p.enabled).collect()
    }

    /// Serialize registry to TOML string (for testing)
    pub fn to_toml(&self) -> Result<String> {
        toml::to_string_pretty(self).map_err(|e| {
            CliError::Plugin(format!("Failed to serialize registry: {}", e))
        })
    }

    /// Deserialize registry from TOML string (for testing)
    pub fn from_toml(content: &str) -> Result<Self> {
        toml::from_str(content).map_err(|e| {
            CliError::Plugin(format!("Failed to parse registry: {}", e))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_entry(name: &str) -> PluginEntry {
        PluginEntry {
            name: name.to_string(),
            version: "0.1.0".to_string(),
            enabled: true,
            source: PluginSource::Local {
                path: PathBuf::from("/tmp/test"),
            },
            install_path: PathBuf::from("/tmp/install"),
            installed_at: "2025-01-01T00:00:00Z".to_string(),
        }
    }

    #[test]
    fn test_add_and_find() {
        let mut reg = PluginRegistry::default();
        reg.add(sample_entry("test-plugin"));
        assert!(reg.find("test-plugin").is_some());
        assert!(reg.find("nonexistent").is_none());
    }

    #[test]
    fn test_add_replaces_existing() {
        let mut reg = PluginRegistry::default();
        reg.add(sample_entry("test-plugin"));
        let mut updated = sample_entry("test-plugin");
        updated.version = "0.2.0".to_string();
        reg.add(updated);
        assert_eq!(reg.plugins.len(), 1);
        assert_eq!(reg.find("test-plugin").unwrap().version, "0.2.0");
    }

    #[test]
    fn test_remove() {
        let mut reg = PluginRegistry::default();
        reg.add(sample_entry("a"));
        reg.add(sample_entry("b"));
        let removed = reg.remove("a");
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().name, "a");
        assert!(reg.find("a").is_none());
        assert!(reg.find("b").is_some());
    }

    #[test]
    fn test_remove_nonexistent() {
        let mut reg = PluginRegistry::default();
        assert!(reg.remove("nope").is_none());
    }

    #[test]
    fn test_enabled_plugins() {
        let mut reg = PluginRegistry::default();
        reg.add(sample_entry("a"));
        let mut disabled = sample_entry("b");
        disabled.enabled = false;
        reg.add(disabled);
        let enabled = reg.enabled_plugins();
        assert_eq!(enabled.len(), 1);
        assert_eq!(enabled[0].name, "a");
    }

    #[test]
    fn test_find_mut() {
        let mut reg = PluginRegistry::default();
        reg.add(sample_entry("test"));
        if let Some(entry) = reg.find_mut("test") {
            entry.enabled = false;
        }
        assert!(!reg.find("test").unwrap().enabled);
    }

    #[test]
    fn test_serialization_roundtrip() {
        let mut reg = PluginRegistry::default();
        reg.add(sample_entry("plugin-a"));
        reg.add(PluginEntry {
            name: "plugin-b".to_string(),
            version: "1.0.0".to_string(),
            enabled: false,
            source: PluginSource::Git {
                url: "https://github.com/test/repo".to_string(),
                rev: Some("abc123".to_string()),
            },
            install_path: PathBuf::from("/home/user/.axum-app-create/plugins/plugin-b"),
            installed_at: "2025-07-15T10:00:00Z".to_string(),
        });

        let toml_str = reg.to_toml().unwrap();
        let restored = PluginRegistry::from_toml(&toml_str).unwrap();
        assert_eq!(reg, restored);
    }
}

#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    fn arb_plugin_source() -> impl Strategy<Value = PluginSource> {
        prop_oneof![
            "[a-z]{1,10}".prop_map(|p| PluginSource::Local {
                path: PathBuf::from(format!("/tmp/{}", p))
            }),
            "[a-z]{1,10}".prop_map(|n| PluginSource::Git {
                url: format!("https://github.com/test/{}", n),
                rev: None,
            }),
            "[a-z]{1,10}".prop_map(|n| PluginSource::Crate {
                name: n.clone(),
                version: "0.1.0".to_string(),
            }),
        ]
    }

    fn arb_plugin_entry() -> impl Strategy<Value = PluginEntry> {
        (
            "[a-z][a-z0-9]{0,9}(-[a-z]{1,5}){0,2}",
            any::<bool>(),
            arb_plugin_source(),
        )
            .prop_map(|(name, enabled, source)| PluginEntry {
                name,
                version: "0.1.0".to_string(),
                enabled,
                source,
                install_path: PathBuf::from("/tmp/install"),
                installed_at: "2025-01-01T00:00:00Z".to_string(),
            })
    }

    /// Operations on the registry
    #[derive(Debug, Clone)]
    enum RegistryOp {
        Add(PluginEntry),
        Remove(String),
    }

    fn arb_registry_op() -> impl Strategy<Value = RegistryOp> {
        prop_oneof![
            arb_plugin_entry().prop_map(RegistryOp::Add),
            "[a-z][a-z0-9]{0,9}(-[a-z]{1,5}){0,2}".prop_map(RegistryOp::Remove),
        ]
    }

    proptest! {
        /// Property 4: Plugin registry round-trip and invariants
        /// (a) Serializing to TOML and deserializing back produces equivalent registry
        /// (b) After adding a plugin, find(name) returns it; after removing, find returns None
        /// **Validates: Requirements 3.7, 3.8, 3.10**
        #[test]
        fn prop_registry_roundtrip_and_invariants(
            ops in prop::collection::vec(arb_registry_op(), 1..20)
        ) {
            let mut reg = PluginRegistry::default();

            for op in &ops {
                match op {
                    RegistryOp::Add(entry) => {
                        reg.add(entry.clone());
                        // Invariant: after add, find returns the entry
                        let found = reg.find(&entry.name);
                        prop_assert!(found.is_some(), "find() should return Some after add");
                        prop_assert_eq!(&found.unwrap().name, &entry.name);
                    }
                    RegistryOp::Remove(name) => {
                        reg.remove(name);
                        // Invariant: after remove, find returns None
                        prop_assert!(reg.find(name).is_none(), "find() should return None after remove");
                    }
                }
            }

            // Round-trip: serialize then deserialize should be equivalent
            let toml_str = reg.to_toml().unwrap();
            let restored = PluginRegistry::from_toml(&toml_str).unwrap();
            prop_assert_eq!(reg, restored);
        }
    }
}
