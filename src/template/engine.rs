// Template engine
//
// This module sets up Handlebars for template rendering.

use crate::error::{CliError, Result};
use crate::template::context::TemplateContext;
use handlebars::Handlebars;

/// Template rendering engine
pub struct TemplateEngine {
    handlebars: Handlebars<'static>,
}

impl TemplateEngine {
    /// Create a new template engine with strict mode enabled
    pub fn new() -> Self {
        let mut handlebars = Handlebars::new();

        // Enable strict mode to catch undefined variables at render time
        handlebars.set_strict_mode(true);

        // Register custom helpers
        register_custom_helpers(&mut handlebars);

        Self { handlebars }
    }

    /// Render a template string with the given context
    ///
    /// # Arguments
    /// * `template_content` - The raw template string (e.g., from include_str!)
    /// * `context` - The template context containing variables
    ///
    /// # Returns
    /// * `Ok(String)` containing the rendered output
    /// * `Err(CliError)` if rendering fails
    pub fn render_template(
        &self,
        template_name: &str,
        template_content: &str,
        context: &TemplateContext,
    ) -> Result<String> {
        self.handlebars
            .render_template(template_content, context)
            .map_err(|e| {
                // Extract line/column information from the error if available
                let error_msg = e.to_string();

                // Try to extract line number from handlebars error
                let line_info = if error_msg.contains("at line") {
                    error_msg
                        .split("at line")
                        .nth(1)
                        .and_then(|s| s.split_whitespace().next())
                        .unwrap_or("unknown")
                } else {
                    "unknown"
                };

                CliError::Template(format!(
                    "❌ 模板渲染失败 / Template rendering failed\n\n\
                     📄 模板名称 / Template name: {}\n\
                     📍 位置 / Line: {}\n\n\
                     💡 修复建议 / Fix:\n\
                     1. 检查模板语法是否正确 / Check template syntax\n\
                     2. 确认所有变量都在上下文中定义 / Ensure all variables are defined in context\n\
                     3. 查看完整错误信息 / See full error message below\n\
                     4. 查看帮助 / View help: axum-app-create --help\n\n\
                     ❌ 错误详情 / Error details:\n\
                     {}",
                    template_name, line_info, error_msg
                ))
            })
    }

    /// Register a template string from memory
    ///
    /// # Arguments
    /// * `name` - Template name for later reference
    /// * `content` - Template content
    pub fn register_template_string(&mut self, name: &str, content: &str) -> Result<()> {
        self.handlebars
            .register_template_string(name, content)
            .map_err(|e| {
                CliError::Template(format!(
                    "❌ 模板注册失败 / Template registration failed\n\n\
                 📄 模板名称 / Template name: {}\n\n\
                 💡 修复建议 / Fix: 检查模板语法 / Check template syntax\n\n\
                 📖 查看帮助 / View help: axum-app-create --help\n\n\
                 ❌ 错误详情 / Error: {}",
                    name, e
                ))
            })
    }

    /// Render a registered template by name
    pub fn render(&self, name: &str, context: &TemplateContext) -> Result<String> {
        self.handlebars.render(name, context).map_err(|e| {
            let error_msg = e.to_string();
            let line_info = if error_msg.contains("at line") {
                error_msg
                    .split("at line")
                    .nth(1)
                    .and_then(|s| s.split_whitespace().next())
                    .unwrap_or("unknown")
            } else {
                "unknown"
            };

            CliError::Template(format!(
                "❌ 模板渲染失败 / Template rendering failed\n\n\
                     📄 模板名称 / Template name: {}\n\
                     📍 位置 / Line: {}\n\n\
                     📖 查看帮助 / View help: axum-app-create --help\n\n\
                     ❌ 错误详情 / Error: {}",
                name, line_info, error_msg
            ))
        })
    }
}

impl Default for TemplateEngine {
    fn default() -> Self {
        Self::new()
    }
}

use handlebars::Renderable;

/// Block helper: renders default content in normal mode.
/// During inheritance, InheritanceProcessor replaces blocks before Handlebars renders.
struct BlockHelper;

impl handlebars::HelperDef for BlockHelper {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &handlebars::Helper<'rc>,
        r: &'reg handlebars::Handlebars<'reg>,
        ctx: &'rc handlebars::Context,
        rc: &mut handlebars::RenderContext<'reg, 'rc>,
        out: &mut dyn handlebars::Output,
    ) -> handlebars::HelperResult {
        if let Some(template) = h.template() {
            template.render(r, ctx, rc, out)?;
        }
        Ok(())
    }
}

/// Override helper: no-op during normal rendering.
/// Override blocks are processed by InheritanceProcessor before Handlebars renders.
struct OverrideHelper;

impl handlebars::HelperDef for OverrideHelper {
    fn call<'reg: 'rc, 'rc>(
        &self,
        _h: &handlebars::Helper<'rc>,
        _r: &'reg handlebars::Handlebars<'reg>,
        _ctx: &'rc handlebars::Context,
        _rc: &mut handlebars::RenderContext<'reg, 'rc>,
        _out: &mut dyn handlebars::Output,
    ) -> handlebars::HelperResult {
        Ok(())
    }
}

/// Register custom Handlebars helpers
fn register_custom_helpers(handlebars: &mut Handlebars) {
    use handlebars::{Output, RenderErrorReason};

    // Helper: block
    // In normal rendering (no inheritance), just renders the default content inside the block.
    // During inheritance, InheritanceProcessor handles block replacement before Handlebars renders.
    handlebars.register_helper("block", Box::new(BlockHelper));

    // Helper: override
    // This is a no-op during normal Handlebars rendering.
    // Override blocks are processed by InheritanceProcessor before rendering.
    handlebars.register_helper("override", Box::new(OverrideHelper));

    // Helper: to_snake_case
    // Converts a string to snake_case
    handlebars.register_helper(
        "to_snake_case",
        Box::new(
            |h: &handlebars::Helper<'_>,
             _r: &handlebars::Handlebars<'_>,
             _: &handlebars::Context,
             _rc: &mut handlebars::RenderContext<'_, '_>,
             out: &mut dyn Output|
             -> handlebars::HelperResult {
                let param = h.param(0).ok_or_else(|| {
                    RenderErrorReason::Other("Missing parameter for to_snake_case".into())
                })?;
                let value = param
                    .value()
                    .as_str()
                    .ok_or_else(|| RenderErrorReason::Other("Parameter must be a string".into()))?;
                let result = to_snake_case(value);
                out.write(&result)?;
                Ok(())
            },
        ),
    );

    // Helper: to_pascal_case
    // Converts a string to PascalCase
    handlebars.register_helper(
        "to_pascal_case",
        Box::new(
            |h: &handlebars::Helper<'_>,
             _r: &handlebars::Handlebars<'_>,
             _: &handlebars::Context,
             _rc: &mut handlebars::RenderContext<'_, '_>,
             out: &mut dyn Output|
             -> handlebars::HelperResult {
                let param = h.param(0).ok_or_else(|| {
                    RenderErrorReason::Other("Missing parameter for to_pascal_case".into())
                })?;
                let value = param
                    .value()
                    .as_str()
                    .ok_or_else(|| RenderErrorReason::Other("Parameter must be a string".into()))?;
                let result = to_pascal_case(value);
                out.write(&result)?;
                Ok(())
            },
        ),
    );

    // Helper: to_upper_camel_case (alias for to_pascal_case)
    handlebars.register_helper(
        "to_upper_camel_case",
        Box::new(
            |h: &handlebars::Helper<'_>,
             _r: &handlebars::Handlebars<'_>,
             _: &handlebars::Context,
             _rc: &mut handlebars::RenderContext<'_, '_>,
             out: &mut dyn Output|
             -> handlebars::HelperResult {
                // Reuse to_pascal_case implementation
                let param = h.param(0).ok_or_else(|| {
                    RenderErrorReason::Other("Missing parameter for to_upper_camel_case".into())
                })?;
                let value = param
                    .value()
                    .as_str()
                    .ok_or_else(|| RenderErrorReason::Other("Parameter must be a string".into()))?;
                let result = to_pascal_case(value);
                out.write(&result)?;
                Ok(())
            },
        ),
    );
}

/// Convert kebab-case to snake_case
fn to_snake_case(name: &str) -> String {
    name.replace('-', "_")
}

/// Convert kebab-case to PascalCase
fn to_pascal_case(name: &str) -> String {
    name.split('-')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_snake_case() {
        assert_eq!(to_snake_case("my-app"), "my_app");
        assert_eq!(to_snake_case("my_axum_app"), "my_axum_app");
        assert_eq!(to_snake_case("myapp"), "myapp");
    }

    #[test]
    fn test_to_pascal_case() {
        assert_eq!(to_pascal_case("my-app"), "MyApp");
        assert_eq!(to_pascal_case("my-axum-app"), "MyAxumApp");
        assert_eq!(to_pascal_case("myapp"), "Myapp");
    }

    #[test]
    fn test_template_engine_creation() {
        let engine = TemplateEngine::new();
        // Basic test - just ensure it doesn't panic
        assert!(engine.handlebars.strict_mode());
    }
}

#[cfg(test)]
mod block_proptests {
    use super::*;
    use crate::config::ProjectConfig;
    use crate::template::context::TemplateContext;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// Property 4: Block refactoring backward compatibility
        /// Rendering a template with {{#block}} markers produces identical output
        /// to rendering the same content without block markers.
        /// Feature: v030-template-and-update, Property 4: Block refactoring backward compatibility
        /// **Validates: Requirements 3.4, 9.1, 9.2, 9.4**
        #[test]
        fn prop_block_helper_is_transparent(content in "[a-zA-Z0-9 ;:=()]{1,80}") {
            // Ensure content doesn't contain Handlebars syntax
            prop_assume!(!content.contains("{{"));
            prop_assume!(!content.contains("}}"));
            // Ensure content has actual non-whitespace characters
            prop_assume!(content.trim().len() > 0);

            let engine = TemplateEngine::new();
            let config = ProjectConfig::default();
            let ctx = TemplateContext::from_config(&config);

            // Render without block markers
            let without_blocks = engine
                .render_template("test_no_block", &content, &ctx)
                .unwrap();

            // Render with block markers wrapping the same content
            let with_blocks = format!("{{{{#block \"test\"}}}}{}{{{{/block}}}}", content);
            let with_blocks_result = engine
                .render_template("test_with_block", &with_blocks, &ctx)
                .unwrap();

            prop_assert_eq!(
                without_blocks, with_blocks_result,
                "Block helper should be transparent"
            );
        }
    }
}
