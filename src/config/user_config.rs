// User-level configuration file loader
//
// This module handles loading and parsing ~/.axum-app-create.toml
// for user-level default settings.

use serde::Deserialize;
use std::path::PathBuf;

/// User-level configuration file
///
/// Loaded from `~/.axum-app-create.toml`
#[derive(Debug, Deserialize, Default, Clone, PartialEq)]
pub struct UserConfig {
    /// Default custom template directory path
    pub template_dir: Option<PathBuf>,
    /// Plugin-specific configuration sections [plugins.<name>]
    #[serde(default)]
    pub plugins: std::collections::HashMap<String, toml::Value>,
}

impl UserConfig {
    /// Load user config from `~/.axum-app-create.toml`
    ///
    /// Returns `Default` if file doesn't exist or is invalid (with warning on invalid TOML).
    pub fn load() -> Self {
        Self::load_from_path(Self::config_path())
    }

    /// Get the config file path
    fn config_path() -> Option<PathBuf> {
        dirs::home_dir().map(|home| home.join(".axum-app-create.toml"))
    }

    /// Load from a specific path (for testing)
    pub(crate) fn load_from_path(path: Option<PathBuf>) -> Self {
        let Some(path) = path else {
            return Self::default();
        };

        if !path.exists() {
            return Self::default();
        }

        match std::fs::read_to_string(&path) {
            Ok(content) => Self::parse(&content),
            Err(e) => {
                eprintln!(
                    "⚠️  警告 / Warning: 无法读取配置文件 / Cannot read config file: {}\n   {}",
                    path.display(),
                    e
                );
                Self::default()
            }
        }
    }

    /// Parse TOML content into UserConfig
    ///
    /// Returns Default on invalid TOML (with warning).
    pub(crate) fn parse(content: &str) -> Self {
        match toml::from_str(content) {
            Ok(config) => config,
            Err(e) => {
                eprintln!(
                    "⚠️  警告 / Warning: 配置文件格式无效 / Invalid config file format\n   {}",
                    e
                );
                Self::default()
            }
        }
    }
}

/// Resolve template_dir with priority: CLI flag > user config > default (None)
pub fn resolve_template_dir(
    cli_flag: Option<PathBuf>,
    user_config: &UserConfig,
) -> Option<PathBuf> {
    cli_flag.or_else(|| user_config.template_dir.clone())
}

/// Merge plugin configuration: manifest defaults + user overrides.
/// User config values take priority over manifest defaults.
/// Warns on unknown config keys (keys in user config not defined in manifest).
/// 合并插件配置：清单默认值 + 用户覆盖值
pub fn merge_plugin_config(
    manifest_defaults: &std::collections::HashMap<String, toml::Value>,
    user_overrides: &toml::Value,
) -> toml::Value {
    let mut merged = toml::map::Map::new();

    // Start with manifest defaults
    for (key, value) in manifest_defaults {
        merged.insert(key.clone(), value.clone());
    }

    // Apply user overrides
    if let toml::Value::Table(user_table) = user_overrides {
        for (key, value) in user_table {
            if !manifest_defaults.contains_key(key) {
                tracing::warn!(
                    "Unknown plugin config key '{}', ignoring / 未知的插件配置键 '{}'，已忽略",
                    key,
                    key
                );
            } else {
                merged.insert(key.clone(), value.clone());
            }
        }
    }

    toml::Value::Table(merged)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_config() {
        let toml = r#"template_dir = "/home/user/.axum-templates""#;
        let config = UserConfig::parse(toml);
        assert_eq!(
            config.template_dir,
            Some(PathBuf::from("/home/user/.axum-templates"))
        );
    }

    #[test]
    fn test_parse_empty_config() {
        let config = UserConfig::parse("");
        assert_eq!(config.template_dir, None);
    }

    #[test]
    fn test_parse_invalid_toml() {
        let config = UserConfig::parse("this is not valid toml {{{}}}");
        assert_eq!(config, UserConfig::default());
    }

    #[test]
    fn test_parse_missing_template_dir() {
        let toml = r#"some_other_key = "value""#;
        // toml crate with deny_unknown_fields would fail, but we don't use it
        // so unknown fields are just ignored
        let config = UserConfig::parse(toml);
        assert_eq!(config.template_dir, None);
    }

    #[test]
    fn test_load_from_nonexistent_path() {
        let config = UserConfig::load_from_path(Some(PathBuf::from("/nonexistent/path.toml")));
        assert_eq!(config, UserConfig::default());
    }

    #[test]
    fn test_load_from_none_path() {
        let config = UserConfig::load_from_path(None);
        assert_eq!(config, UserConfig::default());
    }

    #[test]
    fn test_resolve_template_dir_cli_wins() {
        let cli = Some(PathBuf::from("/cli/path"));
        let user_config = UserConfig {
            template_dir: Some(PathBuf::from("/config/path")),
            plugins: Default::default(),
        };
        assert_eq!(
            resolve_template_dir(cli, &user_config),
            Some(PathBuf::from("/cli/path"))
        );
    }

    #[test]
    fn test_resolve_template_dir_config_fallback() {
        let user_config = UserConfig {
            template_dir: Some(PathBuf::from("/config/path")),
            plugins: Default::default(),
        };
        assert_eq!(
            resolve_template_dir(None, &user_config),
            Some(PathBuf::from("/config/path"))
        );
    }

    #[test]
    fn test_resolve_template_dir_none() {
        let user_config = UserConfig::default();
        assert_eq!(resolve_template_dir(None, &user_config), None);
    }

    #[test]
    fn test_parse_with_plugin_config() {
        let toml = r#"
template_dir = "/home/user/.axum-templates"

[plugins.my-plugin]
schema_path = "src/schema.rs"
playground = false
"#;
        let config = UserConfig::parse(toml);
        assert!(config.plugins.contains_key("my-plugin"));
        let plugin_cfg = &config.plugins["my-plugin"];
        assert_eq!(
            plugin_cfg.get("schema_path").and_then(|v| v.as_str()),
            Some("src/schema.rs")
        );
    }

    #[test]
    fn test_merge_plugin_config() {
        use std::collections::HashMap;
        let mut defaults = HashMap::new();
        defaults.insert(
            "key1".to_string(),
            toml::Value::String("default1".to_string()),
        );
        defaults.insert("key2".to_string(), toml::Value::Boolean(true));

        let mut user_table = toml::map::Map::new();
        user_table.insert(
            "key1".to_string(),
            toml::Value::String("override1".to_string()),
        );
        user_table.insert(
            "unknown_key".to_string(),
            toml::Value::String("ignored".to_string()),
        );
        let user_overrides = toml::Value::Table(user_table);

        let merged = super::merge_plugin_config(&defaults, &user_overrides);
        let table = merged.as_table().unwrap();

        // key1 should be overridden
        assert_eq!(table["key1"].as_str(), Some("override1"));
        // key2 should keep default
        assert_eq!(table["key2"].as_bool(), Some(true));
        // unknown_key should NOT be in merged result
        assert!(!table.contains_key("unknown_key"));
    }
}

#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// Property 12: User config parsing
        /// For any valid TOML with template_dir, parsing produces correct value.
        /// Feature: v030-template-and-update, Property 12: User config parsing
        /// **Validates: Requirements 7.2, 7.3, 7.4**
        #[test]
        fn prop_valid_toml_parsing(path in "[a-zA-Z0-9/_.-]{1,50}") {
            let toml_str = format!(r#"template_dir = "{}""#, path);
            let config = UserConfig::parse(&toml_str);
            prop_assert_eq!(config.template_dir, Some(PathBuf::from(&path)));
        }

        /// Property 12: Invalid TOML returns default without error.
        /// Feature: v030-template-and-update, Property 12: User config parsing
        /// **Validates: Requirements 7.2, 7.3, 7.4**
        #[test]
        fn prop_invalid_toml_returns_default(garbage in "[^\\x00]{0,100}") {
            // Prepend invalid TOML syntax to ensure it's always invalid
            let invalid = format!("{{{{invalid}}}} {}", garbage);
            let config = UserConfig::parse(&invalid);
            prop_assert_eq!(config, UserConfig::default());
        }
    }
}

#[cfg(test)]
mod priority_proptests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// Property 13: Configuration priority
        /// CLI flag > user config > default (None)
        /// Feature: v030-template-and-update, Property 13: Configuration priority
        /// **Validates: Requirements 1.10, 7.5**
        #[test]
        fn prop_config_priority(
            cli_path in proptest::option::of("[a-zA-Z0-9/_.-]{1,30}"),
            config_path in proptest::option::of("[a-zA-Z0-9/_.-]{1,30}"),
        ) {
            let cli_flag = cli_path.as_ref().map(|p| PathBuf::from(p));
            let user_config = UserConfig {
                template_dir: config_path.as_ref().map(|p| PathBuf::from(p)),
                plugins: Default::default(),
            };

            let result = resolve_template_dir(cli_flag.clone(), &user_config);

            if cli_flag.is_some() {
                // CLI flag always wins
                prop_assert_eq!(result, cli_flag);
            } else if user_config.template_dir.is_some() {
                // User config is fallback
                prop_assert_eq!(result, user_config.template_dir);
            } else {
                // Default is None
                prop_assert_eq!(result, None);
            }
        }
    }
}
