// Plugin manager (facade)
//
// Unified entry point for all plugin operations.

use crate::error::{CliError, Result};
use crate::plugin::dependency::DependencyResolver;
use crate::plugin::hooks::{HookExecutor, HookPoint, HookResult, LoadedPlugin, PluginContext};
use crate::plugin::loader::PluginLoader;
use crate::plugin::manifest::ManifestParser;
use crate::plugin::registry::{PluginEntry, PluginRegistry, PluginSource};
use crate::plugin::sandbox::PluginSandbox;
use std::collections::HashMap;

/// Plugin manager: unified entry point for plugin operations
/// 插件管理器：统一的插件操作入口
pub struct PluginManager {
    registry: PluginRegistry,
    loader: PluginLoader,
    loaded_plugins: Vec<LoadedPlugin>,
}

impl PluginManager {
    /// Initialize plugin manager
    /// 初始化插件管理器
    pub fn new() -> Result<Self> {
        let registry = PluginRegistry::load().unwrap_or_else(|e| {
            tracing::warn!(
                "Failed to load plugin registry, using empty / 加载插件注册表失败，使用空注册表: {}",
                e
            );
            PluginRegistry::default()
        });
        let loader = PluginLoader::new()?;
        Ok(Self {
            registry,
            loader,
            loaded_plugins: Vec::new(),
        })
    }

    /// Try to initialize, returning None on failure (for graceful degradation)
    pub fn try_new() -> Option<Self> {
        match Self::new() {
            Ok(mgr) => Some(mgr),
            Err(e) => {
                tracing::warn!(
                    "Plugin system initialization failed, continuing without plugins / \
                     插件系统初始化失败，以无插件模式继续: {}",
                    e
                );
                None
            }
        }
    }

    /// Install a plugin from source
    /// 安装插件
    pub fn install(&mut self, source: PluginSource, interactive: bool) -> Result<()> {
        // Install to cache
        let install_path = self.loader.install(&source)?;

        // Load and validate manifest
        let manifest = ManifestParser::load_from_dir(&install_path)?;

        // Check version compatibility
        crate::plugin::loader::check_compatibility(&manifest)?;

        // Validate dependencies
        let installed_manifests = self.get_installed_manifests();
        DependencyResolver::validate_dependencies(&manifest, &installed_manifests)?;

        // Check permissions for non-local sources
        if !matches!(source, PluginSource::Local { .. }) {
            let confirmed = PluginSandbox::confirm_permissions(&manifest, interactive)?;
            if !confirmed {
                return Err(CliError::Plugin(format!(
                    "Installation cancelled by user / 用户取消安装: {}",
                    manifest.name
                )));
            }
        }

        // Register the plugin
        let entry = PluginEntry {
            name: manifest.name.clone(),
            version: manifest.version.clone(),
            enabled: true,
            source,
            install_path,
            installed_at: chrono::Utc::now().to_rfc3339(),
        };

        self.registry.add(entry);
        self.registry.save()?;

        println!(
            "✅ Plugin '{}' v{} installed successfully / 插件 '{}' v{} 安装成功",
            manifest.name, manifest.version, manifest.name, manifest.version
        );

        Ok(())
    }

    /// Uninstall a plugin
    /// 卸载插件
    pub fn uninstall(&mut self, name: &str, interactive: bool) -> Result<()> {
        // Check if plugin exists
        let entry = self.registry.find(name).ok_or_else(|| {
            CliError::Plugin(format!(
                "Plugin '{}' is not installed / 插件 '{}' 未安装",
                name, name
            ))
        })?;

        // Check if other plugins depend on this one
        let dependents = self.find_dependents(name);
        if !dependents.is_empty() {
            let dep_list = dependents.join(", ");
            if interactive {
                println!(
                    "⚠️  The following plugins depend on '{}': {} / \
                     以下插件依赖 '{}'：{}",
                    name, dep_list, name, dep_list
                );
                let confirm = inquire::Confirm::new("Continue with uninstall? / 继续卸载？")
                    .with_default(false)
                    .prompt()
                    .map_err(|e| CliError::Plugin(format!("Prompt failed: {}", e)))?;
                if !confirm {
                    return Ok(());
                }
            } else {
                return Err(CliError::Plugin(format!(
                    "Cannot uninstall '{}': depended upon by {} / \
                     无法卸载 '{}'：被 {} 依赖",
                    name, dep_list, name, dep_list
                )));
            }
        }

        // Remove cached files (only for non-local sources)
        let install_path = entry.install_path.clone();
        if !matches!(entry.source, PluginSource::Local { .. }) && install_path.exists() {
            std::fs::remove_dir_all(&install_path)?;
        }

        self.registry.remove(name);
        self.registry.save()?;

        println!("✅ Plugin '{}' uninstalled / 插件 '{}' 已卸载", name, name);

        Ok(())
    }

    /// Enable a plugin
    /// 启用插件
    pub fn enable(&mut self, name: &str) -> Result<()> {
        let entry = self.registry.find_mut(name).ok_or_else(|| {
            CliError::Plugin(format!(
                "Plugin '{}' is not installed / 插件 '{}' 未安装",
                name, name
            ))
        })?;
        entry.enabled = true;
        self.registry.save()?;
        println!("✅ Plugin '{}' enabled / 插件 '{}' 已启用", name, name);
        Ok(())
    }

    /// Disable a plugin
    /// 禁用插件
    pub fn disable(&mut self, name: &str) -> Result<()> {
        let entry = self.registry.find_mut(name).ok_or_else(|| {
            CliError::Plugin(format!(
                "Plugin '{}' is not installed / 插件 '{}' 未安装",
                name, name
            ))
        })?;
        entry.enabled = false;
        self.registry.save()?;
        println!("✅ Plugin '{}' disabled / 插件 '{}' 已禁用", name, name);
        Ok(())
    }

    /// List all installed plugins
    /// 列出所有插件
    pub fn list(&self) -> &[PluginEntry] {
        &self.registry.plugins
    }

    /// Get plugin details
    /// 获取插件详情
    pub fn info(&self, name: &str) -> Result<&PluginEntry> {
        self.registry.find(name).ok_or_else(|| {
            CliError::Plugin(format!(
                "Plugin '{}' is not installed / 插件 '{}' 未安装",
                name, name
            ))
        })
    }

    /// Load all enabled plugins in dependency order
    /// 按依赖拓扑排序加载所有启用的插件
    pub fn load_enabled(&mut self) -> Result<()> {
        self.loaded_plugins.clear();

        let enabled = self.registry.enabled_plugins();
        if enabled.is_empty() {
            return Ok(());
        }

        // Load manifests for all enabled plugins
        let mut manifests = HashMap::new();
        let mut plugin_dirs = HashMap::new();

        for entry in &enabled {
            match ManifestParser::load_from_dir(&entry.install_path) {
                Ok(manifest) => {
                    plugin_dirs.insert(entry.name.clone(), entry.install_path.clone());
                    manifests.insert(entry.name.clone(), manifest);
                }
                Err(e) => {
                    tracing::warn!(
                        "Failed to load plugin '{}', skipping / 加载插件 '{}' 失败，跳过: {}",
                        entry.name,
                        entry.name,
                        e
                    );
                }
            }
        }

        // Resolve load order
        let load_order = DependencyResolver::resolve(&manifests)?;

        // Load plugins in order
        for name in &load_order {
            if let Some(dir) = plugin_dirs.get(name) {
                match self.loader.load_from_dir(dir) {
                    Ok(loaded) => {
                        self.loaded_plugins.push(loaded);
                    }
                    Err(e) => {
                        tracing::warn!(
                            "Failed to load plugin '{}', skipping / 加载插件 '{}' 失败，跳过: {}",
                            name,
                            name,
                            e
                        );
                    }
                }
            }
        }

        Ok(())
    }

    /// Get all plugin templates (merged by priority)
    /// 获取所有插件模板
    pub fn get_plugin_templates(&self) -> HashMap<String, String> {
        let mut templates = HashMap::new();
        // Lower priority plugins first, higher priority overrides
        let sorted = HookExecutor::sort_by_priority(&self.loaded_plugins);
        for plugin in sorted.iter().rev() {
            for (path, content) in &plugin.templates {
                templates.insert(path.clone(), content.clone());
            }
        }
        // Then higher priority plugins override
        for plugin in &sorted {
            for (path, content) in &plugin.templates {
                templates.insert(path.clone(), content.clone());
            }
        }
        templates
    }

    /// Execute hooks for a given hook point
    /// 执行钩子
    pub fn execute_hook(&self, hook: HookPoint, context: &mut PluginContext) -> Vec<HookResult> {
        HookExecutor::execute(hook, &self.loaded_plugins, context)
    }

    /// Run a plugin command
    /// 运行插件命令
    pub fn run_command(
        &self,
        plugin_name: &str,
        command_name: &str,
        args: &[String],
    ) -> Result<()> {
        let plugin = self
            .loaded_plugins
            .iter()
            .find(|p| p.manifest.name == plugin_name)
            .ok_or_else(|| {
                CliError::Plugin(format!(
                    "Plugin '{}' is not loaded / 插件 '{}' 未加载",
                    plugin_name, plugin_name
                ))
            })?;

        let cmd_def = plugin
            .manifest
            .capabilities
            .commands
            .iter()
            .find(|c| c.name == command_name)
            .ok_or_else(|| {
                CliError::Plugin(format!(
                    "Command '{}' not found in plugin '{}' / \
                     在插件 '{}' 中未找到命令 '{}'",
                    command_name, plugin_name, plugin_name, command_name
                ))
            })?;

        let script_path = plugin.dir.join(&cmd_def.script);
        if !script_path.exists() {
            return Err(CliError::Plugin(format!(
                "Script not found: {} / 脚本未找到: {}",
                script_path.display(),
                script_path.display()
            )));
        }

        // Execute the script
        let mut cmd = std::process::Command::new("sh");
        cmd.arg(&script_path);
        cmd.args(args);
        cmd.current_dir(&plugin.dir);

        let status = cmd.status().map_err(|e| {
            CliError::Plugin(format!(
                "Failed to execute command '{}' from plugin '{}': {} / \
                 执行插件 '{}' 的命令 '{}' 失败: {}",
                command_name, plugin_name, e, plugin_name, command_name, e
            ))
        })?;

        if !status.success() {
            return Err(CliError::Plugin(format!(
                "Command '{}' from plugin '{}' failed with exit code {} / \
                 插件 '{}' 的命令 '{}' 以退出码 {} 失败",
                command_name,
                plugin_name,
                status.code().unwrap_or(-1),
                plugin_name,
                command_name,
                status.code().unwrap_or(-1)
            )));
        }

        Ok(())
    }

    /// Get loaded plugins reference
    pub fn loaded_plugins(&self) -> &[LoadedPlugin] {
        &self.loaded_plugins
    }

    // --- Private helpers ---

    /// Get manifests of all installed plugins
    fn get_installed_manifests(&self) -> HashMap<String, crate::plugin::manifest::PluginManifest> {
        let mut manifests = HashMap::new();
        for entry in &self.registry.plugins {
            if let Ok(manifest) = ManifestParser::load_from_dir(&entry.install_path) {
                manifests.insert(entry.name.clone(), manifest);
            }
        }
        manifests
    }

    /// Find plugins that depend on the given plugin
    fn find_dependents(&self, name: &str) -> Vec<String> {
        let mut dependents = Vec::new();
        for entry in &self.registry.plugins {
            if !entry.enabled || entry.name == name {
                continue;
            }
            if let Ok(manifest) = ManifestParser::load_from_dir(&entry.install_path)
                && manifest.dependencies.contains_key(name)
            {
                dependents.push(entry.name.clone());
            }
        }
        dependents
    }
}
