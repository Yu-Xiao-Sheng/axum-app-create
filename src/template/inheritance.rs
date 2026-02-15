// Template inheritance processor
//
// This module handles template inheritance via extends/block/override directives.
// - `{{!-- extends: <base_template_path> --}}` in a child template
// - `{{#block "name"}}...{{/block}}` in a base template
// - `{{#override "name"}}...{{/override}}` in a child template

use crate::error::Result;
use std::collections::HashMap;

/// Template inheritance processor
pub struct InheritanceProcessor;

impl InheritanceProcessor {
    /// Parse the `extends` directive from template content
    ///
    /// Looks for `{{!-- extends: <path> --}}` at the beginning of the template.
    /// Returns `Some(base_template_path)` or `None`.
    pub fn parse_extends(content: &str) -> Option<String> {
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            if let Some(rest) = trimmed.strip_prefix("{{!--")
                && let Some(rest) = rest.strip_suffix("--}}")
            {
                let inner = rest.trim();
                if let Some(path) = inner.strip_prefix("extends:") {
                    return Some(path.trim().to_string());
                }
            }
            // Only check the first non-empty line for extends
            break;
        }
        None
    }

    /// Parse override blocks from a child template
    ///
    /// Extracts all `{{#override "name"}}...{{/override}}` blocks.
    /// Returns HashMap<block_name, override_content>.
    pub fn parse_overrides(content: &str) -> HashMap<String, String> {
        let mut overrides = HashMap::new();
        let mut i = 0;
        let bytes = content.as_bytes();
        let len = bytes.len();

        while i < len {
            // Find {{#override "name"}}
            if let Some(start_pos) = content[i..].find("{{#override \"") {
                let abs_start = i + start_pos;
                let after_tag = abs_start + "{{#override \"".len();

                // Find closing quote
                if let Some(quote_end) = content[after_tag..].find('"') {
                    let name = &content[after_tag..after_tag + quote_end];

                    // Find }}
                    let after_name = after_tag + quote_end + 1;
                    if let Some(tag_close) = content[after_name..].find("}}") {
                        let content_start = after_name + tag_close + 2;

                        // Find {{/override}}
                        let end_tag = "{{/override}}";
                        if let Some(end_pos) = content[content_start..].find(end_tag) {
                            let block_content = &content[content_start..content_start + end_pos];
                            overrides.insert(name.to_string(), block_content.to_string());
                            i = content_start + end_pos + end_tag.len();
                            continue;
                        }
                    }
                }
            }
            break;
        }

        overrides
    }

    /// Apply inheritance: replace blocks in base template with child overrides
    ///
    /// For each `{{#block "name"}}...{{/block}}` in the base template:
    /// - If the child has an override for "name", use the override content
    /// - Otherwise, keep the default block content
    ///
    /// Returns warnings for override names that don't match any block.
    pub fn apply_inheritance(
        base_content: &str,
        overrides: &HashMap<String, String>,
    ) -> Result<String> {
        let mut result = String::new();
        let mut used_overrides: std::collections::HashSet<&str> = std::collections::HashSet::new();
        let mut i = 0;

        while i < base_content.len() {
            // Find {{#block "name"}}
            if let Some(start_pos) = base_content[i..].find("{{#block \"") {
                // Append everything before the block tag
                result.push_str(&base_content[i..i + start_pos]);

                let abs_start = i + start_pos;
                let after_tag = abs_start + "{{#block \"".len();

                // Find closing quote
                if let Some(quote_end) = base_content[after_tag..].find('"') {
                    let name = &base_content[after_tag..after_tag + quote_end];

                    // Find }}
                    let after_name = after_tag + quote_end + 1;
                    if let Some(tag_close) = base_content[after_name..].find("}}") {
                        let content_start = after_name + tag_close + 2;

                        // Find {{/block}}
                        let end_tag = "{{/block}}";
                        if let Some(end_pos) = base_content[content_start..].find(end_tag) {
                            let default_content =
                                &base_content[content_start..content_start + end_pos];

                            // Use override if available, otherwise default
                            if let Some(override_content) = overrides.get(name) {
                                result.push_str(override_content);
                                used_overrides.insert(name);
                            } else {
                                result.push_str(default_content);
                            }

                            i = content_start + end_pos + end_tag.len();
                            continue;
                        }
                    }
                }

                // Malformed block tag - just append as-is
                result.push_str(&base_content[abs_start..abs_start + 1]);
                i = abs_start + 1;
            } else {
                // No more blocks, append the rest
                result.push_str(&base_content[i..]);
                break;
            }
        }

        // Warn about unused overrides
        for name in overrides.keys() {
            if !used_overrides.contains(name.as_str()) {
                eprintln!(
                    "⚠️  警告 / Warning: 覆盖块 '{}' 在基础模板中不存在，已忽略 / \
                     Override block '{}' not found in base template, ignored",
                    name, name
                );
            }
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_extends_present() {
        let content = "{{!-- extends: single_mode/src/main.rs.hbs --}}\n\nsome content";
        assert_eq!(
            InheritanceProcessor::parse_extends(content),
            Some("single_mode/src/main.rs.hbs".to_string())
        );
    }

    #[test]
    fn test_parse_extends_absent() {
        let content = "just normal template content\n{{#if something}}";
        assert_eq!(InheritanceProcessor::parse_extends(content), None);
    }

    #[test]
    fn test_parse_extends_with_leading_whitespace() {
        let content = "\n  {{!-- extends: base.hbs --}}\ncontent";
        assert_eq!(
            InheritanceProcessor::parse_extends(content),
            Some("base.hbs".to_string())
        );
    }

    #[test]
    fn test_parse_overrides_single() {
        let content = r#"{{!-- extends: base.hbs --}}
{{#override "imports"}}use custom::lib;{{/override}}"#;
        let overrides = InheritanceProcessor::parse_overrides(content);
        assert_eq!(overrides.len(), 1);
        assert_eq!(overrides["imports"], "use custom::lib;");
    }

    #[test]
    fn test_parse_overrides_multiple() {
        let content = r#"{{#override "imports"}}import1{{/override}}
{{#override "routes"}}route1{{/override}}"#;
        let overrides = InheritanceProcessor::parse_overrides(content);
        assert_eq!(overrides.len(), 2);
        assert_eq!(overrides["imports"], "import1");
        assert_eq!(overrides["routes"], "route1");
    }

    #[test]
    fn test_parse_overrides_none() {
        let content = "no overrides here";
        let overrides = InheritanceProcessor::parse_overrides(content);
        assert!(overrides.is_empty());
    }

    #[test]
    fn test_apply_inheritance_with_override() {
        let base = r#"{{#block "imports"}}default imports{{/block}}
{{#block "routes"}}default routes{{/block}}"#;
        let mut overrides = HashMap::new();
        overrides.insert("imports".to_string(), "custom imports".to_string());

        let result = InheritanceProcessor::apply_inheritance(base, &overrides).unwrap();
        assert!(result.contains("custom imports"));
        assert!(result.contains("default routes"));
        assert!(!result.contains("default imports"));
    }

    #[test]
    fn test_apply_inheritance_no_overrides() {
        let base = r#"{{#block "imports"}}default imports{{/block}}"#;
        let overrides = HashMap::new();

        let result = InheritanceProcessor::apply_inheritance(base, &overrides).unwrap();
        assert_eq!(result, "default imports");
    }

    #[test]
    fn test_apply_inheritance_preserves_surrounding_content() {
        let base = r#"before
{{#block "middle"}}default{{/block}}
after"#;
        let mut overrides = HashMap::new();
        overrides.insert("middle".to_string(), "custom".to_string());

        let result = InheritanceProcessor::apply_inheritance(base, &overrides).unwrap();
        assert_eq!(result, "before\ncustom\nafter");
    }
}

#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// Property 3: Extends directive parsing
        /// For any valid extends directive, parse_extends extracts the correct path.
        /// For content without the directive, parse_extends returns None.
        /// Feature: v030-template-and-update, Property 3: Extends directive parsing
        /// **Validates: Requirements 2.1**
        #[test]
        fn prop_extends_parsing_valid(path in "[a-zA-Z0-9_/.-]{1,40}") {
            let content = format!("{{{{!-- extends: {} --}}}}\nsome content", path);
            let result = InheritanceProcessor::parse_extends(&content);
            prop_assert_eq!(result, Some(path));
        }

        /// Property 3: Content without extends returns None
        /// Feature: v030-template-and-update, Property 3: Extends directive parsing
        /// **Validates: Requirements 2.1**
        #[test]
        fn prop_extends_parsing_absent(content in "[a-zA-Z0-9 \n]{0,100}") {
            // Ensure content doesn't accidentally contain extends directive
            prop_assume!(!content.contains("extends:"));
            let result = InheritanceProcessor::parse_extends(&content);
            prop_assert_eq!(result, None);
        }
    }
}

#[cfg(test)]
mod inheritance_proptests {
    use super::*;
    use proptest::prelude::*;

    fn block_name() -> impl Strategy<Value = String> {
        "[a-z_]{1,10}".prop_map(|s| s)
    }

    fn block_content() -> impl Strategy<Value = String> {
        // Avoid content that contains block/override markers
        "[a-zA-Z0-9 ,;:]{1,30}".prop_map(|s| s)
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// Property 2: Block default content preservation in inheritance
        /// For any base template with named blocks and a child that overrides a subset,
        /// the merged output contains override content for overridden blocks
        /// and default content for non-overridden blocks.
        /// Feature: v030-template-and-update, Property 2: Block default content preservation
        /// **Validates: Requirements 2.2, 2.3, 2.4, 2.9**
        #[test]
        fn prop_block_inheritance(
            blocks in proptest::collection::vec((block_name(), block_content()), 1..5),
            override_indices in proptest::collection::vec(any::<bool>(), 1..5),
        ) {
            // Deduplicate block names
            let mut seen = std::collections::HashSet::new();
            let blocks: Vec<(String, String)> = blocks
                .into_iter()
                .filter(|(name, _)| seen.insert(name.clone()))
                .collect();

            if blocks.is_empty() {
                return Ok(());
            }

            // Build base template with blocks
            let mut base = String::new();
            for (name, default) in &blocks {
                base.push_str(&format!("{{{{#block \"{}\"}}}}{}{{{{/block}}}}\n", name, default));
            }

            // Build overrides for a subset
            let mut overrides = HashMap::new();
            for (i, (name, _)) in blocks.iter().enumerate() {
                let should_override = override_indices.get(i).copied().unwrap_or(false);
                if should_override {
                    overrides.insert(name.clone(), format!("OVERRIDE_{}", name));
                }
            }

            // Ensure at least one block is NOT overridden (strict subset)
            if overrides.len() == blocks.len() && blocks.len() > 1 {
                let first_key = blocks[0].0.clone();
                overrides.remove(&first_key);
            }

            let result = InheritanceProcessor::apply_inheritance(&base, &overrides).unwrap();

            // Verify: overridden blocks have override content
            for (name, _) in &blocks {
                if let Some(override_content) = overrides.get(name) {
                    prop_assert!(
                        result.contains(override_content),
                        "Override content for '{}' should be in result", name
                    );
                } else {
                    // Non-overridden blocks should have default content
                    let (_, default) = blocks.iter().find(|(n, _)| n == name).unwrap();
                    prop_assert!(
                        result.contains(default.as_str()),
                        "Default content for '{}' should be in result", name
                    );
                }
            }
        }
    }
}
