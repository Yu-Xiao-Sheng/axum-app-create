// Plugin security sandbox
//
// Controls filesystem access and permission validation for plugins.

use crate::error::{CliError, Result};
use crate::plugin::manifest::PluginManifest;
use std::path::Path;

/// Plugin security sandbox
/// 插件安全沙箱
pub struct PluginSandbox;

impl PluginSandbox {
    /// Validate that a path is within the allowed scope.
    /// Allowed: project directory or plugin's own cache directory.
    /// 验证路径是否在允许范围内
    pub fn validate_path(path: &Path, project_dir: &Path, plugin_dir: &Path) -> Result<()> {
        // Canonicalize paths for comparison (handle symlinks, .., etc.)
        // If canonicalization fails (path doesn't exist yet), use the raw path
        let check_path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
        let proj = project_dir
            .canonicalize()
            .unwrap_or_else(|_| project_dir.to_path_buf());
        let plug = plugin_dir
            .canonicalize()
            .unwrap_or_else(|_| plugin_dir.to_path_buf());

        if check_path.starts_with(&proj) || check_path.starts_with(&plug) {
            Ok(())
        } else {
            Err(CliError::Plugin(format!(
                "Security violation: path '{}' is outside allowed scope / \
                 安全违规：路径 '{}' 超出允许范围。\
                 Allowed: project dir '{}' or plugin dir '{}' / \
                 允许范围：项目目录 '{}' 或插件目录 '{}'",
                path.display(),
                path.display(),
                project_dir.display(),
                plugin_dir.display(),
                project_dir.display(),
                plugin_dir.display(),
            )))
        }
    }

    /// Check if a plugin has the required permission for an action.
    /// 检查插件是否有执行操作所需的权限
    pub fn check_permissions(manifest: &PluginManifest, action: &str) -> Result<()> {
        match action {
            "network" if !manifest.permissions.network => Err(CliError::Plugin(format!(
                "Plugin '{}' does not have network permission / \
                 插件 '{}' 没有网络访问权限",
                manifest.name, manifest.name
            ))),
            "filesystem" if !manifest.permissions.filesystem => Err(CliError::Plugin(format!(
                "Plugin '{}' does not have filesystem permission / \
                 插件 '{}' 没有文件系统访问权限",
                manifest.name, manifest.name
            ))),
            "exec" if !manifest.permissions.exec => Err(CliError::Plugin(format!(
                "Plugin '{}' does not have exec permission / \
                 插件 '{}' 没有执行外部命令权限",
                manifest.name, manifest.name
            ))),
            _ => Ok(()),
        }
    }

    /// Display permission summary and request user confirmation.
    /// Returns true if confirmed, false if rejected.
    /// 显示权限摘要并请求用户确认
    pub fn confirm_permissions(manifest: &PluginManifest, interactive: bool) -> Result<bool> {
        let perms = &manifest.permissions;
        let has_special_perms = perms.network || perms.filesystem || perms.exec;

        if !has_special_perms {
            return Ok(true); // No special permissions needed
        }

        println!(
            "\n⚠️  Plugin '{}' requests the following permissions / \
             插件 '{}' 请求以下权限:",
            manifest.name, manifest.name
        );
        if perms.network {
            println!("  🌐 Network access / 网络访问");
        }
        if perms.filesystem {
            println!("  📁 Filesystem access (beyond project dir) / 文件系统访问（超出项目目录）");
        }
        if perms.exec {
            println!("  ⚡ Execute external commands / 执行外部命令");
        }

        if !interactive {
            // In non-interactive mode, reject special permissions
            return Err(CliError::Plugin(format!(
                "Plugin '{}' requires special permissions. Use interactive mode to confirm. / \
                 插件 '{}' 需要特殊权限，请使用交互模式确认。",
                manifest.name, manifest.name
            )));
        }

        // In interactive mode, use inquire to ask
        let confirm = inquire::Confirm::new("Grant these permissions? / 授予这些权限？")
            .with_default(false)
            .prompt()
            .map_err(|e| CliError::Plugin(format!("Permission prompt failed: {}", e)))?;

        Ok(confirm)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugin::manifest::{PluginCapabilities, PluginPermissions};
    use std::collections::HashMap;

    fn make_manifest_with_perms(network: bool, filesystem: bool, exec: bool) -> PluginManifest {
        PluginManifest {
            name: "test-plugin".to_string(),
            version: "0.1.0".to_string(),
            description: "test".to_string(),
            min_tool_version: "0.4.0".to_string(),
            author: None,
            license: None,
            repository: None,
            homepage: None,
            keywords: None,
            capabilities: PluginCapabilities::default(),
            dependencies: HashMap::new(),
            config: HashMap::new(),
            permissions: PluginPermissions {
                network,
                filesystem,
                exec,
            },
            priority: 100,
        }
    }

    #[test]
    fn test_validate_path_within_project() {
        let project = Path::new("/tmp/my-project");
        let plugin = Path::new("/tmp/plugins/test");
        assert!(PluginSandbox::validate_path(
            Path::new("/tmp/my-project/src/main.rs"),
            project,
            plugin,
        )
        .is_ok());
    }

    #[test]
    fn test_validate_path_within_plugin() {
        let project = Path::new("/tmp/my-project");
        let plugin = Path::new("/tmp/plugins/test");
        assert!(
            PluginSandbox::validate_path(
                Path::new("/tmp/plugins/test/templates/a.hbs"),
                project,
                plugin,
            )
            .is_ok()
        );
    }

    #[test]
    fn test_validate_path_outside_scope() {
        let project = Path::new("/tmp/my-project");
        let plugin = Path::new("/tmp/plugins/test");
        assert!(PluginSandbox::validate_path(Path::new("/etc/passwd"), project, plugin,).is_err());
    }

    #[test]
    fn test_check_permissions_granted() {
        let manifest = make_manifest_with_perms(true, false, true);
        assert!(PluginSandbox::check_permissions(&manifest, "network").is_ok());
        assert!(PluginSandbox::check_permissions(&manifest, "exec").is_ok());
    }

    #[test]
    fn test_check_permissions_denied() {
        let manifest = make_manifest_with_perms(false, false, false);
        assert!(PluginSandbox::check_permissions(&manifest, "network").is_err());
        assert!(PluginSandbox::check_permissions(&manifest, "filesystem").is_err());
        assert!(PluginSandbox::check_permissions(&manifest, "exec").is_err());
    }

    #[test]
    fn test_check_unknown_permission_allowed() {
        let manifest = make_manifest_with_perms(false, false, false);
        assert!(PluginSandbox::check_permissions(&manifest, "unknown").is_ok());
    }
}

#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        /// Property 13: Filesystem sandbox path validation
        /// Paths within project_dir or plugin_dir are allowed; all others are rejected.
        /// **Validates: Requirements 11.2, 11.3**
        #[test]
        fn prop_sandbox_path_validation(
            subpath in "[a-z]{1,5}(/[a-z]{1,5}){0,3}",
            choice in 0u8..3,
        ) {
            let project_dir = Path::new("/tmp/sandbox-test-project");
            let plugin_dir = Path::new("/tmp/sandbox-test-plugin");

            match choice {
                0 => {
                    // Path inside project dir — should be allowed
                    let path = project_dir.join(&subpath);
                    prop_assert!(
                        PluginSandbox::validate_path(&path, project_dir, plugin_dir).is_ok(),
                        "Path inside project dir should be allowed: {}",
                        path.display()
                    );
                }
                1 => {
                    // Path inside plugin dir — should be allowed
                    let path = plugin_dir.join(&subpath);
                    prop_assert!(
                        PluginSandbox::validate_path(&path, project_dir, plugin_dir).is_ok(),
                        "Path inside plugin dir should be allowed: {}",
                        path.display()
                    );
                }
                _ => {
                    // Path outside both dirs — should be rejected
                    let path = Path::new("/var/outside").join(&subpath);
                    prop_assert!(
                        PluginSandbox::validate_path(&path, project_dir, plugin_dir).is_err(),
                        "Path outside scope should be rejected: {}",
                        path.display()
                    );
                }
            }
        }
    }
}
