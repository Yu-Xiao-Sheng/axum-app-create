// Plugin loader
//
// Discovers and loads plugins from local paths, Git repos, and crates.io.
// This file initially contains only the version compatibility checker.
// Full loader implementation will be added in Task 10.

use crate::error::{CliError, Result};
use crate::plugin::hooks::LoadedPlugin;
use crate::plugin::manifest::{ManifestParser, PluginManifest};
use crate::plugin::registry::PluginSource;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

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
    let tool_ver = semver::Version::parse(current_tool_version)
        .map_err(|e| CliError::Plugin(format!("Invalid tool version: {}", e)))?;
    let min_ver = semver::Version::parse(min_tool_version)
        .map_err(|e| CliError::Plugin(format!("Invalid min_tool_version: {}", e)))?;

    if tool_ver < min_ver {
        return Err(CliError::Plugin(format!(
            "Requires version >= {}, current is {} / 要求版本 >= {}，当前为 {}",
            min_ver, tool_ver, min_ver, tool_ver
        )));
    }

    Ok(())
}

/// Plugin loader: discovers and loads plugins from various sources
/// 插件加载器：从各种来源发现和加载插件
pub struct PluginLoader {
    cache_dir: PathBuf,
}

impl PluginLoader {
    /// Create a new PluginLoader, initializing the cache directory
    pub fn new() -> Result<Self> {
        let home = dirs::home_dir().ok_or_else(|| {
            CliError::Plugin("Cannot determine home directory / 无法确定主目录".to_string())
        })?;
        let cache_dir = home.join(".axum-app-create").join("plugins");
        if !cache_dir.exists() {
            std::fs::create_dir_all(&cache_dir)?;
        }
        Ok(Self { cache_dir })
    }

    /// Create with a custom cache directory (for testing)
    pub fn with_cache_dir(cache_dir: PathBuf) -> Result<Self> {
        if !cache_dir.exists() {
            std::fs::create_dir_all(&cache_dir)?;
        }
        Ok(Self { cache_dir })
    }

    /// Get the cache directory path
    pub fn cache_dir(&self) -> &Path {
        &self.cache_dir
    }

    /// Install a plugin from source to cache directory, returns the install path
    pub fn install(&self, source: &PluginSource) -> Result<PathBuf> {
        match source {
            PluginSource::Local { path } => {
                // For local plugins, validate and return the path directly
                if !path.exists() {
                    return Err(CliError::Plugin(format!(
                        "Plugin path does not exist: {} / 插件路径不存在: {}",
                        path.display(),
                        path.display()
                    )));
                }
                // Validate it has a plugin.toml
                let manifest_path = path.join("plugin.toml");
                if !manifest_path.exists() {
                    return Err(CliError::Plugin(format!(
                        "No plugin.toml found at {} / 在 {} 中未找到 plugin.toml",
                        path.display(),
                        path.display()
                    )));
                }
                Ok(path.clone())
            }
            PluginSource::Git { url, rev } => self.install_git(url, rev.as_deref()),
            PluginSource::Crate { name, version } => {
                // Stub for future crates.io support
                Err(CliError::Plugin(format!(
                    "crates.io plugin installation not yet supported: {}@{} / \
                     crates.io 插件安装尚未支持: {}@{}",
                    name, version, name, version
                )))
            }
        }
    }

    /// Clone a plugin from a Git repository
    fn install_git(&self, url: &str, rev: Option<&str>) -> Result<PathBuf> {
        // Derive a directory name from the URL
        let dir_name = url
            .rsplit('/')
            .next()
            .unwrap_or("plugin")
            .trim_end_matches(".git");
        let dest = self.cache_dir.join(dir_name);

        if dest.exists() {
            // Already cached, optionally update
            tracing::info!(
                "Plugin already cached at {} / 插件已缓存于 {}",
                dest.display(),
                dest.display()
            );
            return Ok(dest);
        }

        tracing::info!("Cloning plugin from {} / 从 {} 克隆插件", url, url);

        let mut builder = git2::build::RepoBuilder::new();
        if let Some(rev_str) = rev {
            builder.branch(rev_str);
        }

        builder.clone(url, &dest).map_err(|e| {
            CliError::Plugin(format!(
                "Failed to clone plugin from '{}': {} / 从 '{}' 克隆插件失败: {}",
                url, e, url, e
            ))
        })?;

        Ok(dest)
    }

    /// Load a plugin from a local directory
    pub fn load_from_dir(&self, dir: &Path) -> Result<LoadedPlugin> {
        let manifest = ManifestParser::load_from_dir(dir)?;

        // Check version compatibility
        check_compatibility(&manifest)?;

        // Load templates if the plugin declares template capability
        let templates = if manifest.capabilities.templates {
            Self::load_templates(dir)?
        } else {
            HashMap::new()
        };

        Ok(LoadedPlugin {
            manifest,
            dir: dir.to_path_buf(),
            templates,
        })
    }

    /// Load template files from a plugin's templates/ directory
    fn load_templates(plugin_dir: &Path) -> Result<HashMap<String, String>> {
        let templates_dir = plugin_dir.join("templates");
        if !templates_dir.exists() {
            return Ok(HashMap::new());
        }

        let mut templates = HashMap::new();
        Self::walk_templates(&templates_dir, &templates_dir, &mut templates)?;
        Ok(templates)
    }

    /// Recursively walk template directory and collect .hbs files
    fn walk_templates(
        base: &Path,
        current: &Path,
        templates: &mut HashMap<String, String>,
    ) -> Result<()> {
        if !current.is_dir() {
            return Ok(());
        }
        for entry in std::fs::read_dir(current)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                Self::walk_templates(base, &path, templates)?;
            } else {
                let rel_path = path
                    .strip_prefix(base)
                    .map_err(|e| CliError::Plugin(format!("Path error: {}", e)))?;
                let content = std::fs::read_to_string(&path)?;
                templates.insert(rel_path.to_string_lossy().to_string(), content);
            }
        }
        Ok(())
    }
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
