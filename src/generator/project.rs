// Project generation orchestration
//
// This module handles the main project generation logic.

use crate::config::ProjectConfig;
use crate::config::ProjectMode;
use crate::error::{CliError, Result};
use crate::template::context::TemplateContext;
use crate::template::engine::TemplateEngine;
use crate::template::resolver::TemplateResolver;
use crate::updater::checksum::ChecksumCalculator;
use crate::updater::metadata::MetadataManager;
use std::path::{Path, PathBuf};

/// Generate a new project with the given configuration
///
/// This function orchestrates the entire project generation process:
/// - Creates project directory
/// - Renders and writes all template files
/// - Initializes git repository
///
/// # Arguments
/// * `project_dir` - Path where the project should be created
/// * `config` - Project configuration
/// * `interactive` - Whether to prompt for user input on conflicts
/// * `force` - Force overwrite if directory exists
///
/// # Returns
/// * `Ok(())` if generation succeeded
/// * `Err(CliError)` if generation failed
pub fn generate_project(
    project_dir: &Path,
    config: &ProjectConfig,
    interactive: bool,
    force: bool,
) -> Result<()> {
    generate_project_with_templates(project_dir, config, interactive, force, None)
}

/// Generate a new project with optional custom template directory
pub fn generate_project_with_templates(
    project_dir: &Path,
    config: &ProjectConfig,
    interactive: bool,
    force: bool,
    template_dir: Option<PathBuf>,
) -> Result<()> {
    use crate::plugin::hooks::{HookPoint, PluginContext};
    use crate::plugin::manager::PluginManager;

    // Initialize plugin system (graceful degradation on failure)
    let mut plugin_manager = PluginManager::try_new();

    // Load enabled plugins
    if let Some(ref mut mgr) = plugin_manager
        && let Err(e) = mgr.load_enabled()
    {
        tracing::warn!(
            "Failed to load plugins, continuing without plugins / \
                 加载插件失败，以无插件模式继续: {}",
            e
        );
        plugin_manager = None;
    }

    // Validate project directory doesn't exist
    if project_dir.exists() {
        // --force flag: delete and recreate
        if force {
            println!(
                "🗑️  --force: 正在删除现有目录 / Deleting existing directory: '{}'",
                project_dir.display()
            );
            std::fs::remove_dir_all(project_dir)?;
        } else if !interactive {
            // In non-interactive mode without --force, fail immediately
            return Err(CliError::Generation(format!(
                "❌ 目录已存在 / Directory already exists: '{}'\n\n\
                 💡 修复建议 / Fix:\n\
                 - 删除现有目录 / Remove existing directory: rm -rf {}\n\
                 - 使用不同的名称 / Use a different name\n\
                 - 使用 --force 标志强制覆盖 / Use --force flag to overwrite\n\
                 - 查看帮助 / View help: axum-app-create --help",
                project_dir.display(),
                project_dir.display()
            )));
        } else {
            // In interactive mode, prompt for action
            println!(
                "\n⚠️  警告 / Warning: 目录已存在 / Directory already exists: '{}'",
                project_dir.display()
            );
            println!("📁 位置 / Location: {}", project_dir.display());
            println!();

            // Use inquire for user choice
            let options = vec![
                "覆盖 / Overwrite - Delete existing directory and regenerate",
                "取消 / Cancel - Abort project generation",
                "重命名 / Rename - Keep existing directory, use different name",
            ];

            let ans = inquire::Select::new("请选择操作 / Choose an action:", options).prompt()?;

            match ans {
                "覆盖 / Overwrite - Delete existing directory and regenerate" => {
                    println!("🗑️  正在删除现有目录 / Deleting existing directory...");
                    std::fs::remove_dir_all(project_dir)?;
                    println!("✓ 已删除 / Deleted");
                }
                "取消 / Cancel - Abort project generation" => {
                    println!("❌ 已取消 / Aborted");
                    return Err(CliError::Generation(
                        "项目生成已取消 / Project generation cancelled by user".to_string(),
                    ));
                }
                "重命名 / Rename - Keep existing directory, use different name" => {
                    println!(
                        "❌ 请使用不同的项目名称重新运行 / Please run again with a different project name"
                    );
                    return Err(CliError::Generation(
                        "请使用不同的项目名称 / Please use a different project name".to_string(),
                    ));
                }
                _ => {
                    return Err(CliError::Generation(
                        "无效选择 / Invalid choice".to_string(),
                    ));
                }
            }
        }
    }

    println!(
        "\n🚀 正在创建项目 / Creating project: {}",
        config.project_name
    );
    println!("📁 位置 / Location: {}", project_dir.display());

    // Create project directory
    if let Err(e) = std::fs::create_dir_all(project_dir) {
        return handle_permission_error(e, project_dir);
    }

    // Create template context
    let ctx = TemplateContext::from_config(config);

    // Create template engine
    let engine = TemplateEngine::new();

    // Execute pre_generate hook
    let mut plugin_context = plugin_manager.as_ref().map(|_| PluginContext {
        tool_version: env!("CARGO_PKG_VERSION").to_string(),
        project_dir: project_dir.to_path_buf(),
        plugin_config: toml::Value::Table(toml::map::Map::new()),
        plugin_dir: PathBuf::new(),
    });

    if let (Some(mgr), Some(pctx)) = (&plugin_manager, &mut plugin_context) {
        let results = mgr.execute_hook(HookPoint::PreGenerate, pctx);
        for r in &results {
            if !r.success {
                tracing::warn!(
                    "Plugin '{}' pre_generate hook failed: {} / 插件 '{}' pre_generate 钩子失败",
                    r.plugin_name,
                    r.error.as_deref().unwrap_or("unknown"),
                    r.plugin_name
                );
            }
        }
    }

    // Execute modify_context hook to extend template context
    if let (Some(mgr), Some(pctx)) = (&plugin_manager, &mut plugin_context) {
        let results = mgr.execute_hook(HookPoint::ModifyContext, pctx);
        for r in &results {
            if r.success && !r.context_additions.is_empty() {
                tracing::info!(
                    "Plugin '{}' added {} context variables / 插件 '{}' 添加了 {} 个上下文变量",
                    r.plugin_name,
                    r.context_additions.len(),
                    r.plugin_name,
                    r.context_additions.len()
                );
            }
        }
    }

    // Collect plugin templates
    let plugin_templates = plugin_manager
        .as_ref()
        .map(|mgr| mgr.get_plugin_templates())
        .unwrap_or_default();

    // Execute modify_templates hook
    let mut extra_templates = std::collections::HashMap::new();
    if let (Some(mgr), Some(pctx)) = (&plugin_manager, &mut plugin_context) {
        let results = mgr.execute_hook(HookPoint::ModifyTemplates, pctx);
        for r in &results {
            if r.success {
                extra_templates.extend(r.template_additions.clone());
            }
        }
    }

    // Merge hook template additions with plugin templates
    let mut all_plugin_templates = plugin_templates;
    all_plugin_templates.extend(extra_templates);

    // Resolve templates (built-in + plugin + optional custom templates)
    let resolver = TemplateResolver::new(template_dir);
    let resolved = if all_plugin_templates.is_empty() {
        resolver.resolve(config.mode, config.ci)?
    } else {
        resolver.resolve_with_plugins(config.mode, config.ci, &all_plugin_templates)?
    };

    // Render and write each template
    println!("\n📝 Generating files:");

    let mut generated_files: Vec<String> = Vec::new();

    for (name, template) in &resolved {
        // Render template
        let rendered = engine.render_template(name, &template.content, &ctx)?;

        // Skip files that render to empty content (conditional templates)
        if rendered.trim().is_empty() {
            continue;
        }

        // Write file
        write_file(project_dir, &template.path, &rendered)?;
        generated_files.push(template.path.clone());

        println!("  ✓ Created {}", template.path);
    }

    // Initialize git repository
    println!("\n🔧 Initializing git repository...");
    super::git::init_git_repo(project_dir)?;

    // Write generation metadata (.axum-app-create.json)
    println!("📋 Writing generation metadata...");
    let file_checksums = ChecksumCalculator::calculate_all(project_dir, &generated_files)?;
    MetadataManager::create(project_dir, config, file_checksums)?;
    println!("  ✓ Created {}", crate::updater::metadata::METADATA_FILE);

    // Update dependencies to latest compatible versions
    println!("📦 Updating dependencies to latest compatible versions...");
    let update_output = std::process::Command::new("cargo")
        .arg("update")
        .current_dir(project_dir)
        .output();
    match update_output {
        Ok(output) if output.status.success() => {
            println!("  ✓ Dependencies updated");
        }
        _ => {
            println!("  ⚠ Could not update dependencies, run `cargo update` manually");
        }
    }

    // Execute post_generate hook
    if let (Some(mgr), Some(pctx)) = (&plugin_manager, &mut plugin_context) {
        let results = mgr.execute_hook(HookPoint::PostGenerate, pctx);
        for r in &results {
            if !r.success {
                tracing::warn!(
                    "Plugin '{}' post_generate hook failed: {} / 插件 '{}' post_generate 钩子失败",
                    r.plugin_name,
                    r.error.as_deref().unwrap_or("unknown"),
                    r.plugin_name
                );
            }
        }
    }

    // Verify workspace Cargo.toml files (Requirement 5.5)
    if config.mode == ProjectMode::Workspace {
        let required_files = [
            "Cargo.toml",
            "api/Cargo.toml",
            "domain/Cargo.toml",
            "infrastructure/Cargo.toml",
            "common/Cargo.toml",
        ];
        for file in &required_files {
            if !project_dir.join(file).exists() {
                return Err(CliError::Generation(format!(
                    "❌ 工作区验证失败 / Workspace verification failed: 缺少文件 / Missing file: {}",
                    file
                )));
            }
        }
        println!("  ✓ Workspace structure verified");
    }

    Ok(())
}

/// Handle permission errors with helpful suggestions
///
/// # Arguments
/// * `error` - The IO error that occurred
/// * `path` - The path where the error occurred
///
/// # Returns
/// * `Err(CliError)` with helpful permission error message
fn handle_permission_error(error: std::io::Error, path: &Path) -> Result<()> {
    if error.kind() == std::io::ErrorKind::PermissionDenied {
        Err(CliError::Io(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            format!(
                "❌ 权限拒绝 / Permission denied: 无法访问目录 / cannot access directory: '{}'\n\n\
                 💡 修复建议 / Fix:\n\
                 1. 使用 --force 标志强制覆盖 / Use --force flag to overwrite\n\
                 2. 切换到用户目录 / Switch to user directory: cd ~\n\
                 3. 使用临时目录 / Use temp directory: /tmp/my-project\n\
                 4. 检查目录权限 / Check directory permissions: ls -la {}\n\
                 5. 使用sudo（不推荐）/ Use sudo (not recommended): sudo axum-app-create\n\n\
                 📖 查看帮助 / View help: axum-app-create --help\n\n\
                 ❌ 错误详情 / Error: {}",
                path.display(),
                path.parent()
                    .map(|p| p.display().to_string())
                    .unwrap_or_else(|| ".".to_string()),
                error
            ),
        )))
    } else {
        Err(CliError::Io(error))
    }
}

/// Write a file to the project directory
///
/// # Arguments
/// * `project_dir` - Project root directory
/// * `relative_path` - Relative path from project root (e.g., "src/main.rs")
/// * `content` - File content to write
///
/// # Returns
/// * `Ok(())` if write succeeded
/// * `Err(CliError)` if write failed
pub fn write_file(project_dir: &Path, relative_path: &str, content: &str) -> Result<()> {
    let file_path = project_dir.join(relative_path);

    // Create parent directories if needed
    if let Some(parent) = file_path.parent()
        && let Err(e) = std::fs::create_dir_all(parent)
    {
        return handle_permission_error(e, &file_path);
    }

    // Write file
    if let Err(e) = std::fs::write(&file_path, content) {
        return handle_permission_error(e, &file_path);
    }

    Ok(())
}

/// Ensure a directory exists in the project
///
/// # Arguments
/// * `project_dir` - Project root directory
/// * `relative_path` - Relative path to directory (e.g., "src/handlers")
///
/// # Returns
/// * `Ok(())` if directory exists or was created
/// * `Err(CliError)` if directory creation failed
pub fn ensure_dir(project_dir: &Path, relative_path: &str) -> Result<()> {
    let dir_path = project_dir.join(relative_path);
    if let Err(e) = std::fs::create_dir_all(&dir_path) {
        return handle_permission_error(e, &dir_path);
    }
    Ok(())
}

/// Get success message for project generation
pub fn get_success_message(project_dir: &Path, project_name: &str) -> String {
    use chrono::Utc;

    let generation_time = Utc::now().format("%Y-%m-%d %H:%M:%S UTC");

    format!(
        r#"
✨ ══════════════════════════════════════════════════════ ✓
✨                                                        ✨
✨  Project '{}' created successfully!                      ✨
✨                                                        ✨
✨ ══════════════════════════════════════════════════════ ✓

📂 Location:     {}
🕐 Generated:    {}

═════════════════════════════════════════════════════════

🚀 Quick Start:

  $ cd {}
  $ cargo run

═════════════════════════════════════════════════════════

🧪 Test your API:

  # Health check
  $ curl http://127.0.0.1:8080/health

  # Expected response: {{"status":"ok"}}

═════════════════════════════════════════════════════════

📖 Next Steps:

  1. Review the generated code in src/
  2. Customize your configuration in .env
  3. Add new endpoints in src/handlers/
  4. Run tests: cargo test
  5. Build for release: cargo build --release

═════════════════════════════════════════════════════════

💡 Need help?

  $ cd {} && cargo run --help
  $ axum-app-create --help

Happy hacking! 🦀

"#,
        project_name,
        project_dir.display(),
        generation_time,
        project_name,
        project_name
    )
}

/// Get success message for workspace mode project generation
pub fn get_success_message_with_config(project_dir: &Path, config: &ProjectConfig) -> String {
    use chrono::Utc;

    let generation_time = Utc::now().format("%Y-%m-%d %H:%M:%S UTC");
    let project_name = &config.project_name;

    let mode_info = match config.mode {
        ProjectMode::Workspace => "\n📦 Mode:         Workspace (multi-crate)\n\
             📁 Crates:       api, domain, infrastructure, common"
            .to_string(),
        ProjectMode::Single => "\n📦 Mode:         Single package".to_string(),
    };

    let ci_info = if config.ci {
        "\n🔄 CI/CD:        GitHub Actions workflow generated (.github/workflows/ci.yml)"
    } else {
        ""
    };

    format!(
        r#"
✨ ══════════════════════════════════════════════════════ ✓
✨                                                        ✨
✨  Project '{}' created successfully!                      ✨
✨                                                        ✨
✨ ══════════════════════════════════════════════════════ ✓

📂 Location:     {}
🕐 Generated:    {}{}{}

═════════════════════════════════════════════════════════

🚀 Quick Start:

  $ cd {}
  $ cargo run

═════════════════════════════════════════════════════════

🧪 Test your API:

  # Health check
  $ curl http://127.0.0.1:8080/health

  # Expected response: {{"status":"ok"}}

═════════════════════════════════════════════════════════

Happy hacking! 🦀

"#,
        project_name,
        project_dir.display(),
        generation_time,
        mode_info,
        ci_info,
        project_name
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_write_file() {
        let temp_dir = TempDir::new().unwrap();
        let content = "Hello, World!";

        let result = write_file(temp_dir.path(), "test.txt", content);

        assert!(result.is_ok());
        assert!(temp_dir.path().join("test.txt").exists());
    }

    #[test]
    fn test_ensure_dir() {
        let temp_dir = TempDir::new().unwrap();

        let result = ensure_dir(temp_dir.path(), "nested/dir/test");

        assert!(result.is_ok());
        assert!(temp_dir.path().join("nested/dir/test").exists());
    }

    #[test]
    fn test_generate_project_creates_all_files() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path().join("my-test-app");
        let mut config = ProjectConfig::default();
        config.project_name = "my-test-app".to_string();

        let result = generate_project(&project_dir, &config, false, false);

        if let Err(e) = &result {
            eprintln!("Generation error: {:?}", e);
        }

        assert!(result.is_ok());
        assert!(project_dir.exists());

        // Verify key files were created
        assert!(project_dir.join("Cargo.toml").exists());
        assert!(project_dir.join("src/main.rs").exists());
        assert!(project_dir.join("src/lib.rs").exists());
        assert!(project_dir.join("src/config.rs").exists());
        assert!(project_dir.join("src/handlers/health.rs").exists());
        assert!(project_dir.join(".env.example").exists());
        assert!(project_dir.join(".gitignore").exists());
        assert!(project_dir.join("README.md").exists());
    }

    #[test]
    fn test_generate_project_creates_metadata() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path().join("meta-test-app");
        let mut config = ProjectConfig::default();
        config.project_name = "meta-test-app".to_string();

        generate_project(&project_dir, &config, false, false).unwrap();

        // Verify metadata file exists
        let metadata_path = project_dir.join(crate::updater::metadata::METADATA_FILE);
        assert!(metadata_path.exists(), "Metadata file should be created");

        // Verify metadata is valid JSON
        let metadata = crate::updater::metadata::MetadataManager::read(&project_dir).unwrap();
        assert_eq!(metadata.config.project_name, "meta-test-app");
        assert!(!metadata.file_checksums.is_empty(), "Should have checksums");

        // Verify .gitignore contains metadata file entry
        let gitignore = std::fs::read_to_string(project_dir.join(".gitignore")).unwrap();
        assert!(
            gitignore.contains(".axum-app-create.json"),
            ".gitignore should exclude metadata file"
        );
    }
}

#[cfg(test)]
mod proptests {
    use super::*;
    use crate::config::{DatabaseOption, FeatureSet};
    use crate::updater::checksum::ChecksumCalculator;
    use crate::updater::metadata::MetadataManager;
    use proptest::prelude::*;
    use tempfile::TempDir;

    fn arb_project_config() -> impl Strategy<Value = ProjectConfig> {
        (
            prop_oneof![Just(ProjectMode::Single), Just(ProjectMode::Workspace)],
            prop_oneof![
                Just(DatabaseOption::None),
                Just(DatabaseOption::PostgreSQL),
                Just(DatabaseOption::SQLite),
                Just(DatabaseOption::Both),
            ],
            prop::bool::ANY,
            prop::bool::ANY,
            prop::bool::ANY,
        )
            .prop_map(|(mode, db, auth, biz_error, ci)| {
                let features = FeatureSet {
                    database: db,
                    authentication: auth,
                    logging: true,
                    biz_error,
                };
                let mut config = ProjectConfig {
                    project_name: "prop-test-app".to_string(),
                    features,
                    mode,
                    ci,
                    ..Default::default()
                };
                if db.is_enabled() {
                    config.database = Some(crate::config::DatabaseConfig::default());
                }
                if auth {
                    config.authentication = Some(crate::config::AuthConfig::default());
                }
                if biz_error {
                    config.biz_error = Some(crate::config::BizErrorConfig::default());
                }
                config
            })
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(20))]

        /// Property 10: Metadata contains checksums for all generated files
        /// After project generation, .axum-app-create.json should contain a checksum
        /// entry for every generated file, and each checksum should match the SHA-256
        /// of the corresponding file's content on disk.
        /// Feature: v030-template-and-update, Property 10: Metadata checksum completeness
        /// **Validates: Requirements 5.2, 5.4**
        #[test]
        fn prop_metadata_checksum_completeness(config in arb_project_config()) {
            let temp_dir = TempDir::new().unwrap();
            let project_dir = temp_dir.path().join("prop-test-app");

            let result = generate_project(&project_dir, &config, false, false);
            prop_assert!(result.is_ok(), "Generation failed: {:?}", result.err());

            // Read metadata
            let metadata = MetadataManager::read(&project_dir).unwrap();

            // Every file in checksums should exist on disk and match
            for (file_path, stored_checksum) in &metadata.file_checksums {
                let full_path = project_dir.join(file_path);
                prop_assert!(
                    full_path.exists(),
                    "File in metadata does not exist on disk: {}",
                    file_path
                );

                let content = std::fs::read(&full_path).unwrap();
                let actual_checksum = ChecksumCalculator::calculate(&content);
                prop_assert_eq!(
                    stored_checksum,
                    &actual_checksum,
                    "Checksum mismatch for file: {}",
                    file_path
                );
            }

            // Metadata should have at least the core files
            prop_assert!(
                metadata.file_checksums.contains_key("Cargo.toml"),
                "Metadata should contain Cargo.toml checksum"
            );
        }
    }
}
