// Template exporter
//
// This module exports built-in templates to the filesystem,
// allowing users to customize them as a starting point.

use crate::config::ProjectMode;
use crate::error::{CliError, Result};
use crate::template::templates::{
    get_ci_templates, get_single_mode_templates, get_workspace_mode_templates,
};
use std::path::Path;

/// Template exporter: writes built-in templates to filesystem
pub struct TemplateExporter;

impl TemplateExporter {
    /// Export built-in templates for the given mode to the output directory
    ///
    /// Each template is written with its registry key + `.hbs` extension,
    /// preserving the directory structure.
    pub fn export(mode: ProjectMode, output_dir: &Path) -> Result<()> {
        if output_dir.exists()
            && std::fs::read_dir(output_dir)
                .map(|mut d| d.next().is_some())
                .unwrap_or(false)
        {
            return Err(CliError::Config(format!(
                "❌ 输出目录非空 / Output directory is not empty: '{}'\n\
                 💡 请使用空目录或不存在的目录 / Please use an empty or non-existent directory",
                output_dir.display()
            )));
        }

        std::fs::create_dir_all(output_dir)?;

        let templates = match mode {
            ProjectMode::Single => get_single_mode_templates(),
            ProjectMode::Workspace => get_workspace_mode_templates(),
        };

        // Also include CI templates
        let ci_templates = get_ci_templates();

        let mut count = 0;

        for (key, tf) in templates.into_iter().chain(ci_templates.into_iter()) {
            let file_name = format!("{}.hbs", key);
            let file_path = output_dir.join(&file_name);

            if let Some(parent) = file_path.parent() {
                std::fs::create_dir_all(parent)?;
            }

            std::fs::write(&file_path, tf.content)?;
            count += 1;
        }

        println!(
            "✅ 已导出 {} 个模板到 / Exported {} templates to: {}",
            count,
            count,
            output_dir.display()
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_export_single_mode() {
        let temp = TempDir::new().unwrap();
        let output = temp.path().join("templates");

        TemplateExporter::export(ProjectMode::Single, &output).unwrap();

        assert!(output.join("Cargo.toml.hbs").exists());
        assert!(output.join("src/main.rs.hbs").exists());
        assert!(output.join("src/lib.rs.hbs").exists());
        assert!(output.join(".github/workflows/ci.yml.hbs").exists());
    }

    #[test]
    fn test_export_workspace_mode() {
        let temp = TempDir::new().unwrap();
        let output = temp.path().join("templates");

        TemplateExporter::export(ProjectMode::Workspace, &output).unwrap();

        assert!(output.join("Cargo.toml.hbs").exists());
        assert!(output.join("api/Cargo.toml.hbs").exists());
        assert!(output.join("domain/Cargo.toml.hbs").exists());
    }

    #[test]
    fn test_export_non_empty_dir_fails() {
        let temp = TempDir::new().unwrap();
        std::fs::write(temp.path().join("existing.txt"), "data").unwrap();

        let result = TemplateExporter::export(ProjectMode::Single, temp.path());
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod proptests {
    use super::*;
    use crate::template::custom_loader::CustomTemplateLoader;
    use crate::template::templates::{
        get_ci_templates, get_single_mode_templates, get_workspace_mode_templates,
    };
    use proptest::prelude::*;
    use tempfile::TempDir;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// Property 11: Template export round-trip
        /// Export templates, reload as custom, verify identical content.
        /// Feature: v030-template-and-update, Property 11: Template export round-trip
        /// **Validates: Requirements 1.7, 1.8**
        #[test]
        fn prop_template_export_roundtrip(
            mode in prop_oneof![Just(ProjectMode::Single), Just(ProjectMode::Workspace)]
        ) {
            let temp = TempDir::new().unwrap();
            let output = temp.path().join("exported");

            // Export
            TemplateExporter::export(mode, &output).unwrap();

            // Reload as custom templates
            let reloaded = CustomTemplateLoader::load(&output).unwrap();

            // Get original built-in templates
            let mut originals = match mode {
                ProjectMode::Single => get_single_mode_templates(),
                ProjectMode::Workspace => get_workspace_mode_templates(),
            };
            originals.extend(get_ci_templates());

            // Every original template should be in the reloaded set with identical content
            for (key, tf) in &originals {
                prop_assert!(
                    reloaded.contains_key(*key),
                    "Reloaded set missing template: {}",
                    key
                );
                prop_assert_eq!(
                    &reloaded[*key],
                    tf.content,
                    "Content mismatch for template: {}",
                    key
                );
            }

            // Reloaded set should have same count
            prop_assert_eq!(
                reloaded.len(),
                originals.len(),
                "Template count mismatch: reloaded={}, originals={}",
                reloaded.len(),
                originals.len()
            );
        }
    }
}
