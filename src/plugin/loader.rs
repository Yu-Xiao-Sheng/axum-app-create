// Plugin loader
//
// Discovers and loads plugins from local paths, Git repos, and crates.io.
// This file initially contains only the version compatibility checker.
// Full loader implementation will be added in Task 10.

use crate::error::{CliError, Result};
use crate::plugin::manifest::PluginManifest;

/// Current tool version (from Cargo.toml)
pub const TOOL_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Check if a plugin's min_tool_version is compatible with the current tool version.
/// Returns Ok(()) if compatible, Err if the plugin requires a newer tool version.
/// 检查插件的 min_tool_version 是否与当前工具版本兼容
pub fn check_compatibility(manifest: &PluginManifest) -> Result<()> {
    let tool_ver = semver::Version::parse(TOOL_VERSION).map_err(|e| {
        CliError::Plugin(format!(
            "Invalid tool version '{}' / 无效的工具版本: {}",
            TOOL_VERSION, e
        ))
    })?;
    let min_ver = semver::Version::parse(&manifest.min_tool_version).map_err(|e| {
        CliError::Plugin(format!(
            "Invalid min_tool_version '{}' in plugin '{}' / 插件 '{}' 的 min_tool_version 无效: {}",
            manifest.min_tool_version, manifest.name, manifest.name, e
        ))
    })?;

    if tool_ver < min_ver {
        return Err(CliError::Plugin(format!(
            "Plugin '{}' requires tool version >= {}, but current version is {} / \
             插件 '{}' 要求工具版本 >= {}，但当前版本为 {}",
            manifest.name, min_ver, tool_ver, manifest.name, min_ver, tool_ver
        )));
    }

    Ok(())
}

/// Check version compatibility with explicit tool version (for testing)
/// 使用显式工具版本检查兼容性（用于测试）
pub fn check_compatibility_with_version(
    min_tool_version: &str,
    current_tool_version: &str,
) -> Result<()> {
    let tool_ver = semver::Version::parse(current_tool_version).map_err(|e| {
        CliError::Plugin(format!("Invalid tool version: {}", e))
    })?;
    let min_ver = semver::Version::parse(min_tool_version).map_err(|e| {
        CliError::Plugin(format!("Invalid min_tool_version: {}", e))
    })?;

    if tool_ver < min_ver {
        return Err(CliError::Plugin(format!(
            "Requires version >= {}, current is {} / 要求版本 >= {}，当前为 {}",
            min_ver, tool_ver, min_ver, tool_ver
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compatible_same_version() {
        assert!(check_compatibility_with_version("0.4.0", "0.4.0").is_ok());
    }

    #[test]
    fn test_compatible_newer_tool() {
        assert!(check_compatibility_with_version("0.4.0", "0.5.0").is_ok());
        assert!(check_compatibility_with_version("0.4.0", "1.0.0").is_ok());
    }

    #[test]
    fn test_incompatible_older_tool() {
        assert!(check_compatibility_with_version("0.5.0", "0.4.0").is_err());
        assert!(check_compatibility_with_version("1.0.0", "0.4.0").is_err());
    }

    #[test]
    fn test_invalid_version_string() {
        assert!(check_compatibility_with_version("not-a-version", "0.4.0").is_err());
        assert!(check_compatibility_with_version("0.4.0", "bad").is_err());
    }
}

#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    fn arb_semver() -> impl Strategy<Value = (u32, u32, u32)> {
        (0u32..50, 0u32..50, 0u32..50)
    }

    proptest! {
        /// Property 3: Version compatibility check
        /// For any pair of semantic versions, the check returns Ok when tool >= min,
        /// and Err otherwise.
        /// **Validates: Requirements 2.7, 2.8**
        #[test]
        fn prop_version_compatibility(
            min in arb_semver(),
            tool in arb_semver(),
        ) {
            let min_str = format!("{}.{}.{}", min.0, min.1, min.2);
            let tool_str = format!("{}.{}.{}", tool.0, tool.1, tool.2);

            let min_ver = semver::Version::new(min.0 as u64, min.1 as u64, min.2 as u64);
            let tool_ver = semver::Version::new(tool.0 as u64, tool.1 as u64, tool.2 as u64);

            let result = check_compatibility_with_version(&min_str, &tool_str);

            if tool_ver >= min_ver {
                prop_assert!(result.is_ok(), "Expected Ok for tool {} >= min {}", tool_str, min_str);
            } else {
                prop_assert!(result.is_err(), "Expected Err for tool {} < min {}", tool_str, min_str);
            }
        }
    }
}
