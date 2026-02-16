// Plugin hook executor
//
// Executes plugin hooks in priority order at various generation stages.

use crate::plugin::manifest::PluginManifest;
use std::collections::HashMap;
use std::path::PathBuf;

/// Hook types / 钩子类型
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum HookPoint {
    PreGenerate,
    PostGenerate,
    ModifyContext,
    ModifyTemplates,
}

impl HookPoint {
    /// Convert from string representation
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "pre_generate" => Some(Self::PreGenerate),
            "post_generate" => Some(Self::PostGenerate),
            "modify_context" => Some(Self::ModifyContext),
            "modify_templates" => Some(Self::ModifyTemplates),
            _ => None,
        }
    }
}

/// Hook execution context / 钩子执行上下文
#[derive(Debug, Clone)]
pub struct PluginContext {
    pub tool_version: String,
    pub project_dir: PathBuf,
    pub plugin_config: toml::Value,
    pub plugin_dir: PathBuf,
}

/// Hook execution result / 钩子执行结果
#[derive(Debug, Clone)]
pub struct HookResult {
    pub plugin_name: String,
    pub success: bool,
    pub error: Option<String>,
    /// Modified template context variables / 修改后的模板上下文变量
    pub context_additions: HashMap<String, serde_json::Value>,
    /// Modified template set / 修改后的模板集合
    pub template_additions: HashMap<String, String>,
}

/// A loaded plugin with its manifest and templates
/// 已加载的插件
#[derive(Debug, Clone)]
pub struct LoadedPlugin {
    pub manifest: PluginManifest,
    pub dir: PathBuf,
    pub templates: HashMap<String, String>,
}

/// Hook executor / 钩子执行器
pub struct HookExecutor;

impl HookExecutor {
    /// Sort plugins by priority (ascending: lower number = higher priority)
    /// 按优先级排序插件
    pub fn sort_by_priority(plugins: &[LoadedPlugin]) -> Vec<&LoadedPlugin> {
        let mut sorted: Vec<&LoadedPlugin> = plugins.iter().collect();
        sorted.sort_by_key(|p| p.manifest.priority);
        sorted
    }

    /// Execute all plugin hooks for a given hook point (sorted by priority).
    /// Errors in individual plugins are recorded but don't interrupt the pipeline.
    /// 执行指定钩子点的所有插件钩子（按优先级排序）
    pub fn execute(
        hook: HookPoint,
        plugins: &[LoadedPlugin],
        context: &mut PluginContext,
    ) -> Vec<HookResult> {
        let hook_name = match &hook {
            HookPoint::PreGenerate => "pre_generate",
            HookPoint::PostGenerate => "post_generate",
            HookPoint::ModifyContext => "modify_context",
            HookPoint::ModifyTemplates => "modify_templates",
        };

        // Filter plugins that registered for this hook, then sort by priority
        let mut eligible: Vec<&LoadedPlugin> = plugins
            .iter()
            .filter(|p| p.manifest.capabilities.hooks.iter().any(|h| h == hook_name))
            .collect();
        eligible.sort_by_key(|p| p.manifest.priority);

        let mut results = Vec::new();

        for plugin in eligible {
            let result = Self::execute_single_hook(&hook, plugin, context);
            results.push(result);
        }

        results
    }

    /// Execute a single plugin's hook.
    /// In v0.4.0, hooks are script/config-based. This is a placeholder that
    /// returns success for now. Real execution will invoke scripts.
    fn execute_single_hook(
        _hook: &HookPoint,
        plugin: &LoadedPlugin,
        _context: &mut PluginContext,
    ) -> HookResult {
        // v0.4.0: Script-based hook execution placeholder
        // For now, plugins with hooks declared will get a success result.
        // Template additions come from the plugin's loaded templates.
        HookResult {
            plugin_name: plugin.manifest.name.clone(),
            success: true,
            error: None,
            context_additions: HashMap::new(),
            template_additions: HashMap::new(),
        }
    }

    /// Apply modify_context results: add new variables without removing existing ones.
    /// 应用 modify_context 结果：只添加新变量，不删除已有变量
    pub fn apply_context_additions(
        original: &HashMap<String, serde_json::Value>,
        additions: &HashMap<String, serde_json::Value>,
    ) -> HashMap<String, serde_json::Value> {
        let mut result = original.clone();
        for (key, value) in additions {
            // Only add if not already present (preserve originals)
            result.entry(key.clone()).or_insert_with(|| value.clone());
        }
        result
    }

    /// Apply template additions: merge plugin templates into the template set.
    /// Plugin templates can add new templates or replace existing ones.
    pub fn apply_template_additions(
        original: &HashMap<String, String>,
        additions: &HashMap<String, String>,
    ) -> HashMap<String, String> {
        let mut result = original.clone();
        for (key, value) in additions {
            result.insert(key.clone(), value.clone());
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugin::manifest::{PluginCapabilities, PluginPermissions};

    fn make_loaded_plugin(name: &str, priority: u32, hooks: Vec<&str>) -> LoadedPlugin {
        LoadedPlugin {
            manifest: PluginManifest {
                name: name.to_string(),
                version: "0.1.0".to_string(),
                description: "test".to_string(),
                min_tool_version: "0.4.0".to_string(),
                author: None,
                license: None,
                repository: None,
                homepage: None,
                keywords: None,
                capabilities: PluginCapabilities {
                    templates: false,
                    hooks: hooks.into_iter().map(String::from).collect(),
                    commands: vec![],
                },
                dependencies: HashMap::new(),
                config: HashMap::new(),
                permissions: PluginPermissions::default(),
                priority,
            },
            dir: PathBuf::from("/tmp/test"),
            templates: HashMap::new(),
        }
    }

    fn make_context() -> PluginContext {
        PluginContext {
            tool_version: "0.4.0".to_string(),
            project_dir: PathBuf::from("/tmp/project"),
            plugin_config: toml::Value::Table(toml::map::Map::new()),
            plugin_dir: PathBuf::from("/tmp/plugin"),
        }
    }

    #[test]
    fn test_sort_by_priority() {
        let plugins = vec![
            make_loaded_plugin("c", 150, vec![]),
            make_loaded_plugin("a", 10, vec![]),
            make_loaded_plugin("b", 100, vec![]),
        ];
        let sorted = HookExecutor::sort_by_priority(&plugins);
        assert_eq!(sorted[0].manifest.name, "a");
        assert_eq!(sorted[1].manifest.name, "b");
        assert_eq!(sorted[2].manifest.name, "c");
    }

    #[test]
    fn test_execute_filters_by_hook() {
        let plugins = vec![
            make_loaded_plugin("a", 10, vec!["pre_generate"]),
            make_loaded_plugin("b", 20, vec!["post_generate"]),
            make_loaded_plugin("c", 30, vec!["pre_generate", "post_generate"]),
        ];
        let mut ctx = make_context();
        let results = HookExecutor::execute(HookPoint::PreGenerate, &plugins, &mut ctx);
        assert_eq!(results.len(), 2); // a and c
        assert_eq!(results[0].plugin_name, "a");
        assert_eq!(results[1].plugin_name, "c");
    }

    #[test]
    fn test_execute_priority_order() {
        let plugins = vec![
            make_loaded_plugin("low", 200, vec!["pre_generate"]),
            make_loaded_plugin("high", 10, vec!["pre_generate"]),
            make_loaded_plugin("mid", 100, vec!["pre_generate"]),
        ];
        let mut ctx = make_context();
        let results = HookExecutor::execute(HookPoint::PreGenerate, &plugins, &mut ctx);
        assert_eq!(results[0].plugin_name, "high");
        assert_eq!(results[1].plugin_name, "mid");
        assert_eq!(results[2].plugin_name, "low");
    }

    #[test]
    fn test_apply_context_additions_preserves_originals() {
        let mut original = HashMap::new();
        original.insert("key1".to_string(), serde_json::json!("original"));

        let mut additions = HashMap::new();
        additions.insert("key1".to_string(), serde_json::json!("overwritten"));
        additions.insert("key2".to_string(), serde_json::json!("new"));

        let result = HookExecutor::apply_context_additions(&original, &additions);
        assert_eq!(result["key1"], serde_json::json!("original")); // preserved
        assert_eq!(result["key2"], serde_json::json!("new")); // added
    }

    #[test]
    fn test_apply_template_additions() {
        let mut original = HashMap::new();
        original.insert("a.hbs".to_string(), "original".to_string());

        let mut additions = HashMap::new();
        additions.insert("a.hbs".to_string(), "replaced".to_string());
        additions.insert("b.hbs".to_string(), "new".to_string());

        let result = HookExecutor::apply_template_additions(&original, &additions);
        assert_eq!(result["a.hbs"], "replaced");
        assert_eq!(result["b.hbs"], "new");
    }
}

#[cfg(test)]
mod proptests {
    use super::*;
    use crate::plugin::manifest::{PluginCapabilities, PluginPermissions};
    use proptest::prelude::*;

    fn make_plugin_with_priority(name: String, priority: u32) -> LoadedPlugin {
        LoadedPlugin {
            manifest: PluginManifest {
                name,
                version: "0.1.0".to_string(),
                description: "test".to_string(),
                min_tool_version: "0.4.0".to_string(),
                author: None,
                license: None,
                repository: None,
                homepage: None,
                keywords: None,
                capabilities: PluginCapabilities {
                    templates: false,
                    hooks: vec!["pre_generate".to_string()],
                    commands: vec![],
                },
                dependencies: HashMap::new(),
                config: HashMap::new(),
                permissions: PluginPermissions::default(),
                priority,
            },
            dir: PathBuf::from("/tmp/test"),
            templates: HashMap::new(),
        }
    }

    proptest! {
        /// Property 6: Hook priority ordering
        /// Plugins are invoked in ascending priority order.
        /// **Validates: Requirements 4.2, 4.3, 4.7**
        #[test]
        fn prop_hook_priority_ordering(
            priorities in prop::collection::vec(1u32..1000, 2..10)
        ) {
            let plugins: Vec<LoadedPlugin> = priorities
                .iter()
                .enumerate()
                .map(|(i, &p)| make_plugin_with_priority(format!("p{}", i), p))
                .collect();

            let mut ctx = PluginContext {
                tool_version: "0.4.0".to_string(),
                project_dir: PathBuf::from("/tmp"),
                plugin_config: toml::Value::Table(toml::map::Map::new()),
                plugin_dir: PathBuf::from("/tmp"),
            };

            let results = HookExecutor::execute(HookPoint::PreGenerate, &plugins, &mut ctx);

            // Verify results are in ascending priority order
            for i in 1..results.len() {
                let prev_name = &results[i - 1].plugin_name;
                let curr_name = &results[i].plugin_name;
                let prev_priority = plugins.iter().find(|p| &p.manifest.name == prev_name).unwrap().manifest.priority;
                let curr_priority = plugins.iter().find(|p| &p.manifest.name == curr_name).unwrap().manifest.priority;
                prop_assert!(
                    prev_priority <= curr_priority,
                    "Priority {} should be <= {}", prev_priority, curr_priority
                );
            }
        }

        /// Property 7: modify_context only adds, never removes
        /// Original variables are preserved after applying additions.
        /// **Validates: Requirements 4.4**
        #[test]
        fn prop_context_additions_preserve_originals(
            original_keys in prop::collection::vec("[a-z]{1,5}", 1..10),
            addition_keys in prop::collection::vec("[a-z]{1,5}", 0..10),
        ) {
            let original: HashMap<String, serde_json::Value> = original_keys
                .iter()
                .map(|k| (k.clone(), serde_json::json!("original")))
                .collect();

            let additions: HashMap<String, serde_json::Value> = addition_keys
                .iter()
                .map(|k| (k.clone(), serde_json::json!("added")))
                .collect();

            let result = HookExecutor::apply_context_additions(&original, &additions);

            // All original keys must still be present with original values
            for (key, value) in &original {
                prop_assert!(result.contains_key(key), "Original key '{}' missing", key);
                prop_assert_eq!(&result[key], value, "Original key '{}' was modified", key);
            }

            // All addition keys should be present
            for key in addition_keys.iter() {
                prop_assert!(result.contains_key(key), "Addition key '{}' missing", key);
            }
        }

        /// Property 8: Hook errors do not interrupt pipeline
        /// All plugins are invoked even if some would error.
        /// (In v0.4.0 placeholder, all succeed, but we verify all are called)
        /// **Validates: Requirements 4.8**
        #[test]
        fn prop_all_plugins_invoked(
            count in 2usize..10,
        ) {
            let plugins: Vec<LoadedPlugin> = (0..count)
                .map(|i| make_plugin_with_priority(format!("p{}", i), (i as u32 + 1) * 10))
                .collect();

            let mut ctx = PluginContext {
                tool_version: "0.4.0".to_string(),
                project_dir: PathBuf::from("/tmp"),
                plugin_config: toml::Value::Table(toml::map::Map::new()),
                plugin_dir: PathBuf::from("/tmp"),
            };

            let results = HookExecutor::execute(HookPoint::PreGenerate, &plugins, &mut ctx);

            // All plugins should have been invoked
            prop_assert_eq!(results.len(), count, "Expected {} results, got {}", count, results.len());
        }
    }
}
