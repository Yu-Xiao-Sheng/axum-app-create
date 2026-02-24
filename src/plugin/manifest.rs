// Plugin manifest parser
//
// Handles parsing, validation, and serialization of plugin.toml manifest files.

use crate::error::{CliError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Default hook priority (lower = higher priority)
fn default_priority() -> u32 {
    100
}

/// Plugin manifest: describes plugin metadata and capabilities
/// 插件清单：描述插件的元数据和能力
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PluginManifest {
    /// 插件名称（kebab-case）/ Plugin name
    pub name: String,
    /// 语义化版本 / Semantic version
    pub version: String,
    /// 插件描述 / Description
    pub description: String,
    /// 最低兼容的 CLI_Tool 版本 / Minimum compatible tool version
    pub min_tool_version: String,
    /// 作者（可选）/ Author (optional)
    #[serde(default)]
    pub author: Option<String>,
    /// 许可证（可选）/ License (optional)
    #[serde(default)]
    pub license: Option<String>,
    /// 仓库地址（可选）/ Repository URL (optional)
    #[serde(default)]
    pub repository: Option<String>,
    /// 主页（可选）/ Homepage (optional)
    #[serde(default)]
    pub homepage: Option<String>,
    /// 关键词（可选）/ Keywords (optional)
    #[serde(default)]
    pub keywords: Option<Vec<String>>,
    /// 能力声明 / Capability declarations
    #[serde(default)]
    pub capabilities: PluginCapabilities,
    /// 插件依赖 / Plugin dependencies
    #[serde(default)]
    pub dependencies: HashMap<String, String>,
    /// 插件配置定义 / Plugin config definitions
    #[serde(default)]
    pub config: HashMap<String, toml::Value>,
    /// 权限声明 / Permission declarations
    #[serde(default)]
    pub permissions: PluginPermissions,
    /// 钩子优先级（默认 100，越小越优先）/ Hook priority
    #[serde(default = "default_priority")]
    pub priority: u32,
}

/// Plugin capability declarations
/// 插件能力声明
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct PluginCapabilities {
    /// 是否提供模板集 / Provides template sets
    #[serde(default)]
    pub templates: bool,
    /// 钩子列表 / Hook list (e.g. ["pre_generate", "post_generate"])
    #[serde(default)]
    pub hooks: Vec<String>,
    /// 自定义命令列表 / Custom command list
    #[serde(default)]
    pub commands: Vec<PluginCommandDef>,
}

/// Plugin command definition
/// 插件命令定义
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PluginCommandDef {
    /// 命令名称 / Command name
    pub name: String,
    /// 命令描述 / Command description
    pub description: String,
    /// 命令执行的脚本路径（相对于插件目录）/ Script path relative to plugin dir
    pub script: String,
}

/// Plugin permission declarations
/// 插件权限声明
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct PluginPermissions {
    /// 网络访问 / Network access
    #[serde(default)]
    pub network: bool,
    /// 文件系统访问（超出项目目录）/ Filesystem access beyond project dir
    #[serde(default)]
    pub filesystem: bool,
    /// 执行外部命令 / Execute external commands
    #[serde(default)]
    pub exec: bool,
}

/// Plugin manifest parser
/// 插件清单解析器
pub struct ManifestParser;

impl ManifestParser {
    /// Parse manifest from TOML string
    /// 从 TOML 字符串解析清单
    pub fn parse(content: &str) -> Result<PluginManifest> {
        let manifest: PluginManifest = toml::from_str(content).map_err(|e| {
            CliError::Plugin(format!(
                "Failed to parse plugin.toml / 解析 plugin.toml 失败: {}",
                e
            ))
        })?;
        Self::validate(&manifest)?;
        Ok(manifest)
    }

    /// Serialize manifest to TOML string
    /// 将清单序列化为 TOML 字符串
    pub fn serialize(manifest: &PluginManifest) -> Result<String> {
        toml::to_string_pretty(manifest).map_err(|e| {
            CliError::Plugin(format!(
                "Failed to serialize plugin manifest / 序列化插件清单失败: {}",
                e
            ))
        })
    }

    /// Validate required fields in manifest
    /// 验证清单必填字段
    pub fn validate(manifest: &PluginManifest) -> Result<()> {
        let mut missing = Vec::new();

        if manifest.name.is_empty() {
            missing.push("name");
        }
        if manifest.version.is_empty() {
            missing.push("version");
        }
        if manifest.description.is_empty() {
            missing.push("description");
        }
        if manifest.min_tool_version.is_empty() {
            missing.push("min_tool_version");
        }

        if !missing.is_empty() {
            return Err(CliError::Plugin(format!(
                "Plugin manifest missing required fields / 插件清单缺少必填字段: {}",
                missing.join(", ")
            )));
        }

        Ok(())
    }

    /// Load manifest from plugin directory
    /// 从插件目录加载清单
    pub fn load_from_dir(plugin_dir: &Path) -> Result<PluginManifest> {
        let manifest_path = plugin_dir.join("plugin.toml");
        if !manifest_path.exists() {
            return Err(CliError::Plugin(format!(
                "plugin.toml not found in {} / 在 {} 中未找到 plugin.toml",
                plugin_dir.display(),
                plugin_dir.display()
            )));
        }
        let content = std::fs::read_to_string(&manifest_path)?;
        Self::parse(&content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_toml() -> &'static str {
        r#"
name = "test-plugin"
version = "0.1.0"
description = "A test plugin"
min_tool_version = "0.4.0"

[capabilities]
templates = true
hooks = ["pre_generate"]

[permissions]
exec = true
"#
    }

    #[test]
    fn test_parse_valid_manifest() {
        let manifest = ManifestParser::parse(sample_toml()).unwrap();
        assert_eq!(manifest.name, "test-plugin");
        assert_eq!(manifest.version, "0.1.0");
        assert_eq!(manifest.description, "A test plugin");
        assert_eq!(manifest.min_tool_version, "0.4.0");
        assert!(manifest.capabilities.templates);
        assert_eq!(manifest.capabilities.hooks, vec!["pre_generate"]);
        assert!(manifest.permissions.exec);
        assert!(!manifest.permissions.network);
        assert_eq!(manifest.priority, 100);
    }

    #[test]
    fn test_parse_missing_required_field() {
        let toml = r#"
name = "test"
version = "0.1.0"
description = ""
min_tool_version = "0.4.0"
"#;
        let result = ManifestParser::parse(toml);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("description"));
    }

    #[test]
    fn test_parse_missing_name_field() {
        // TOML deserialization will fail if `name` key is absent entirely
        let toml = r#"
version = "0.1.0"
description = "test"
min_tool_version = "0.4.0"
"#;
        let result = ManifestParser::parse(toml);
        assert!(result.is_err());
    }

    #[test]
    fn test_serialize_roundtrip() {
        let manifest = ManifestParser::parse(sample_toml()).unwrap();
        let serialized = ManifestParser::serialize(&manifest).unwrap();
        let deserialized = ManifestParser::parse(&serialized).unwrap();
        assert_eq!(manifest, deserialized);
    }

    #[test]
    fn test_manifest_with_dependencies() {
        let toml = r#"
name = "dep-plugin"
version = "1.0.0"
description = "Plugin with deps"
min_tool_version = "0.4.0"

[dependencies]
other-plugin = ">=0.1.0"
"#;
        let manifest = ManifestParser::parse(toml).unwrap();
        assert_eq!(
            manifest.dependencies.get("other-plugin").unwrap(),
            ">=0.1.0"
        );
    }

    #[test]
    fn test_manifest_with_commands() {
        let toml = r#"
name = "cmd-plugin"
version = "0.1.0"
description = "Plugin with commands"
min_tool_version = "0.4.0"

[[capabilities.commands]]
name = "gen-schema"
description = "Generate schema"
script = "scripts/gen.sh"
"#;
        let manifest = ManifestParser::parse(toml).unwrap();
        assert_eq!(manifest.capabilities.commands.len(), 1);
        assert_eq!(manifest.capabilities.commands[0].name, "gen-schema");
    }

    #[test]
    fn test_manifest_with_config() {
        let toml = r#"
name = "cfg-plugin"
version = "0.1.0"
description = "Plugin with config"
min_tool_version = "0.4.0"

[config]
schema_path = "src/schema.rs"
playground = true
"#;
        let manifest = ManifestParser::parse(toml).unwrap();
        assert_eq!(manifest.config.len(), 2);
    }

    #[test]
    fn test_load_from_dir_missing() {
        let result = ManifestParser::load_from_dir(Path::new("/nonexistent/path"));
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    /// Generate a valid kebab-case name
    fn arb_kebab_name() -> impl Strategy<Value = String> {
        "[a-z][a-z0-9]{0,9}(-[a-z][a-z0-9]{0,5}){0,2}".prop_map(|s| s)
    }

    /// Generate a valid semver string
    fn arb_semver() -> impl Strategy<Value = String> {
        (0u32..100, 0u32..100, 0u32..100).prop_map(|(ma, mi, p)| format!("{}.{}.{}", ma, mi, p))
    }

    /// Generate a non-empty description
    fn arb_description() -> impl Strategy<Value = String> {
        "[A-Za-z ]{1,30}".prop_map(|s| s)
    }

    /// Generate arbitrary PluginPermissions
    fn arb_permissions() -> impl Strategy<Value = PluginPermissions> {
        (any::<bool>(), any::<bool>(), any::<bool>()).prop_map(|(n, f, e)| PluginPermissions {
            network: n,
            filesystem: f,
            exec: e,
        })
    }

    /// Generate arbitrary PluginCapabilities (no commands to keep TOML round-trip simple)
    fn arb_capabilities() -> impl Strategy<Value = PluginCapabilities> {
        (
            any::<bool>(),
            prop::collection::vec(
                prop::sample::select(vec![
                    "pre_generate".to_string(),
                    "post_generate".to_string(),
                    "modify_context".to_string(),
                    "modify_templates".to_string(),
                ]),
                0..=3,
            ),
        )
            .prop_map(|(templates, hooks)| PluginCapabilities {
                templates,
                hooks,
                commands: vec![],
            })
    }

    /// Generate arbitrary PluginManifest
    fn arb_manifest() -> impl Strategy<Value = PluginManifest> {
        (
            arb_kebab_name(),
            arb_semver(),
            arb_description(),
            arb_semver(),
            arb_capabilities(),
            arb_permissions(),
            1u32..200,
        )
            .prop_map(
                |(
                    name,
                    version,
                    description,
                    min_tool_version,
                    capabilities,
                    permissions,
                    priority,
                )| {
                    PluginManifest {
                        name,
                        version,
                        description,
                        min_tool_version,
                        author: None,
                        license: None,
                        repository: None,
                        homepage: None,
                        keywords: None,
                        capabilities,
                        dependencies: HashMap::new(),
                        config: HashMap::new(),
                        permissions,
                        priority,
                    }
                },
            )
    }

    /// Generate a TOML string with specific required fields removed
    fn build_toml_missing_fields(
        include_name: bool,
        include_version: bool,
        include_description: bool,
        include_min_tool_version: bool,
    ) -> String {
        let mut parts = Vec::new();
        if include_name {
            parts.push("name = \"test-plugin\"".to_string());
        }
        if include_version {
            parts.push("version = \"0.1.0\"".to_string());
        }
        if include_description {
            parts.push("description = \"A test plugin\"".to_string());
        }
        if include_min_tool_version {
            parts.push("min_tool_version = \"0.4.0\"".to_string());
        }
        parts.join("\n")
    }

    proptest! {
        /// Property 1: Plugin manifest serialization round-trip
        /// For any valid PluginManifest, serialize to TOML then deserialize back
        /// SHALL produce an equivalent struct.
        /// **Validates: Requirements 1.7, 1.8, 1.9**
        #[test]
        fn prop_manifest_roundtrip(manifest in arb_manifest()) {
            let serialized = ManifestParser::serialize(&manifest).unwrap();
            let deserialized = ManifestParser::parse(&serialized).unwrap();
            prop_assert_eq!(manifest, deserialized);
        }

        /// Property 2: Plugin manifest required field validation
        /// For any TOML string missing one or more required fields,
        /// parsing SHALL return an error.
        /// **Validates: Requirements 1.2, 1.5**
        #[test]
        fn prop_manifest_missing_required_fields(
            has_name in any::<bool>(),
            has_version in any::<bool>(),
            has_description in any::<bool>(),
            has_min_tool_version in any::<bool>(),
        ) {
            // At least one field must be missing for this test to be meaningful
            prop_assume!(!(has_name && has_version && has_description && has_min_tool_version));

            let toml_str = build_toml_missing_fields(
                has_name,
                has_version,
                has_description,
                has_min_tool_version,
            );
            let result = ManifestParser::parse(&toml_str);
            prop_assert!(result.is_err(), "Expected error when required fields are missing, got Ok");
        }
    }
}
