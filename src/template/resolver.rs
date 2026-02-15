// Template resolver
//
// This module merges built-in templates with custom templates,
// handling override priority and template inheritance.

use crate::config::ProjectMode;
use crate::error::{CliError, Result};
use crate::template::custom_loader::CustomTemplateLoader;
use crate::template::inheritance::InheritanceProcessor;
use crate::template::templates::{
    get_ci_templates, get_single_mode_templates, get_workspace_mode_templates,
};
use std::collections::HashMap;
use std::path::PathBuf;

/// A resolved template ready for rendering
#[derive(Debug, Clone)]
pub struct ResolvedTemplate {
    /// Relative path in generated project (e.g. "src/main.rs")
    pub path: String,
    /// Final template content (Handlebars syntax, inheritance processed)
    pub content: String,
    /// Whether the file is executable
    pub executable: bool,
}

/// Template resolver: merges built-in and custom templates
pub struct TemplateResolver {
    custom_template_dir: Option<PathBuf>,
}

impl TemplateResolver {
    pub fn new(custom_template_dir: Option<PathBuf>) -> Self {
        Self {
            custom_template_dir,
        }
    }

    /// Resolve the final template set
    ///
    /// 1. Load built-in templates (based on ProjectMode)
    /// 2. If custom template dir exists, load custom templates
    /// 3. Custom templates override matching paths, new paths are added
    /// 4. Process template inheritance (extends directives) - done later in task 6
    pub fn resolve(
        &self,
        mode: ProjectMode,
        ci_enabled: bool,
    ) -> Result<HashMap<String, ResolvedTemplate>> {
        // Step 1: Load built-in templates
        let mut builtin = match mode {
            ProjectMode::Single => get_single_mode_templates(),
            ProjectMode::Workspace => get_workspace_mode_templates(),
        };

        if ci_enabled {
            builtin.extend(get_ci_templates());
        }

        // Convert built-in templates to ResolvedTemplate
        let mut resolved: HashMap<String, ResolvedTemplate> = builtin
            .into_iter()
            .map(|(key, tf)| {
                (
                    key.to_string(),
                    ResolvedTemplate {
                        path: tf.path.to_string(),
                        content: tf.content.to_string(),
                        executable: tf.executable,
                    },
                )
            })
            .collect();

        // Step 2: Load and merge custom templates
        if let Some(ref custom_dir) = self.custom_template_dir {
            let custom_templates = CustomTemplateLoader::load(custom_dir)?;

            // Get built-in template contents for inheritance lookups
            let builtin_contents = Self::get_builtin_templates(mode, ci_enabled);

            for (key, content) in custom_templates {
                // Check if this template uses inheritance
                if let Some(base_path) = InheritanceProcessor::parse_extends(&content) {
                    // Find the base template
                    let base_content = builtin_contents.get(&base_path).ok_or_else(|| {
                        CliError::Template(format!(
                            "❌ 模板继承错误 / Template inheritance error\n\
                             📄 子模板 / Child template: {}\n\
                             📄 基础模板不存在 / Base template not found: {}",
                            key, base_path
                        ))
                    })?;

                    // Parse overrides from child and apply to base
                    let overrides = InheritanceProcessor::parse_overrides(&content);
                    let merged = InheritanceProcessor::apply_inheritance(base_content, &overrides)?;

                    let path = resolved
                        .get(&key)
                        .map(|t| t.path.clone())
                        .unwrap_or_else(|| key.clone());

                    resolved.insert(
                        key,
                        ResolvedTemplate {
                            path,
                            content: merged,
                            executable: false,
                        },
                    );
                } else {
                    // No extends directive - full replacement
                    let path = resolved
                        .get(&key)
                        .map(|t| t.path.clone())
                        .unwrap_or_else(|| key.clone());

                    resolved.insert(
                        key,
                        ResolvedTemplate {
                            path,
                            content,
                            executable: false,
                        },
                    );
                }
            }
        }

        Ok(resolved)
    }

    /// Get the built-in templates as a HashMap<key, content> for inheritance lookups
    pub fn get_builtin_templates(mode: ProjectMode, ci_enabled: bool) -> HashMap<String, String> {
        let mut builtin = match mode {
            ProjectMode::Single => get_single_mode_templates(),
            ProjectMode::Workspace => get_workspace_mode_templates(),
        };
        if ci_enabled {
            builtin.extend(get_ci_templates());
        }
        builtin
            .into_iter()
            .map(|(k, v)| (k.to_string(), v.content.to_string()))
            .collect()
    }
}

/// Merge two template sets: custom overrides built-in, new paths are added.
/// This is a pure function for property testing.
pub fn merge_template_sets(
    builtin: &HashMap<String, String>,
    custom: &HashMap<String, String>,
) -> HashMap<String, String> {
    let mut result = builtin.clone();
    for (key, content) in custom {
        result.insert(key.clone(), content.clone());
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge_override() {
        let mut builtin = HashMap::new();
        builtin.insert("a".to_string(), "builtin_a".to_string());
        builtin.insert("b".to_string(), "builtin_b".to_string());

        let mut custom = HashMap::new();
        custom.insert("a".to_string(), "custom_a".to_string());

        let result = merge_template_sets(&builtin, &custom);
        assert_eq!(result["a"], "custom_a");
        assert_eq!(result["b"], "builtin_b");
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_merge_add_new() {
        let mut builtin = HashMap::new();
        builtin.insert("a".to_string(), "builtin_a".to_string());

        let mut custom = HashMap::new();
        custom.insert("c".to_string(), "custom_c".to_string());

        let result = merge_template_sets(&builtin, &custom);
        assert_eq!(result.len(), 2);
        assert_eq!(result["a"], "builtin_a");
        assert_eq!(result["c"], "custom_c");
    }

    #[test]
    fn test_resolver_no_custom_dir() {
        let resolver = TemplateResolver::new(None);
        let result = resolver.resolve(ProjectMode::Single, false).unwrap();
        assert!(!result.is_empty());
        assert!(result.contains_key("Cargo.toml"));
        assert!(result.contains_key("src/main.rs"));
    }

    #[test]
    fn test_resolver_with_custom_dir() {
        let temp = tempfile::TempDir::new().unwrap();
        std::fs::write(temp.path().join("Cargo.toml.hbs"), "custom cargo content").unwrap();

        let resolver = TemplateResolver::new(Some(temp.path().to_path_buf()));
        let result = resolver.resolve(ProjectMode::Single, false).unwrap();

        assert_eq!(result["Cargo.toml"].content, "custom cargo content");
        // Other built-in templates should still be present
        assert!(result.contains_key("src/main.rs"));
    }

    #[test]
    fn test_resolver_with_empty_custom_dir() {
        let temp = tempfile::TempDir::new().unwrap();
        let resolver = TemplateResolver::new(Some(temp.path().to_path_buf()));
        let result = resolver.resolve(ProjectMode::Single, false).unwrap();
        // Should be same as no custom dir
        assert!(result.contains_key("Cargo.toml"));
    }
}

#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::collection::hash_map;
    use proptest::prelude::*;

    fn template_key() -> impl Strategy<Value = String> {
        "[a-z][a-z0-9_/]{0,20}".prop_map(|s| s)
    }

    fn template_content() -> impl Strategy<Value = String> {
        "[a-zA-Z0-9 _]{1,50}".prop_map(|s| s)
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// Property 1: Template resolution override and merge
        /// For any set of built-in and custom templates, the resolved set should:
        /// (a) use custom content for overlapping paths
        /// (b) include custom-only paths
        /// (c) retain built-in-only paths
        /// Total count = unique paths across both sets
        /// Feature: v030-template-and-update, Property 1: Template resolution override and merge
        /// **Validates: Requirements 1.2, 1.3, 2.5**
        #[test]
        fn prop_template_resolution_merge(
            builtin in hash_map(template_key(), template_content(), 1..10),
            custom in hash_map(template_key(), template_content(), 0..10),
        ) {
            let result = merge_template_sets(&builtin, &custom);

            // (a) Overlapping paths use custom content
            for (key, content) in &custom {
                prop_assert_eq!(&result[key], content, "Custom should override for key: {}", key);
            }

            // (b) Custom-only paths are included
            for key in custom.keys() {
                prop_assert!(result.contains_key(key), "Custom-only key missing: {}", key);
            }

            // (c) Built-in-only paths are retained
            for (key, content) in &builtin {
                if !custom.contains_key(key) {
                    prop_assert_eq!(&result[key], content, "Built-in should be retained for key: {}", key);
                }
            }

            // Total count = unique paths
            let mut all_keys: std::collections::HashSet<&String> = builtin.keys().collect();
            all_keys.extend(custom.keys());
            prop_assert_eq!(result.len(), all_keys.len(), "Total count should equal unique paths");
        }
    }
}
