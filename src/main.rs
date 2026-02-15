// axum-app-create: A CLI tool to scaffold Axum web applications
//
// This tool generates new Axum projects with sensible defaults and optional features.
// v0.3.0 introduces subcommands: new, init-template, update

use axum_app_create::cli::{is_non_interactive, prompts::prompt_project_config};
use axum_app_create::config::user_config::{UserConfig, resolve_template_dir};
use axum_app_create::config::{DatabaseOption, Preset, ProjectMode};
use axum_app_create::error::CliError;
use axum_app_create::generator::project::get_success_message_with_config;
use axum_app_create::template::exporter::TemplateExporter;
use axum_app_create::updater::engine::UpdateEngine;
use axum_app_create::utils::rust_toolchain::check_rust_toolchain;
use clap::{Parser, Subcommand};
use std::path::{Path, PathBuf};

/// Simple CLI tool to scaffold Axum web applications
#[derive(Parser, Debug)]
#[command(name = "axum-app-create")]
#[command(about = "Scaffold a new Axum web application", long_about = None)]
#[command(version = "0.3.0")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    // Top-level args for backward compatibility (no subcommand = `new`)
    /// Project name (positional argument)
    #[arg(value_name = "PROJECT_NAME")]
    project_name: Option<String>,

    /// Author name for generated project
    #[arg(long)]
    author: Option<String>,

    /// Database support: none, postgresql, sqlite, or both
    #[arg(long, value_name = "TYPE")]
    database: Option<String>,

    /// Enable JWT authentication
    #[arg(long)]
    auth: bool,

    /// Enable biz-error integration
    #[arg(long)]
    biz_error: bool,

    /// Default log level: trace, debug, info, warn, error
    #[arg(long, value_name = "LEVEL")]
    log_level: Option<String>,

    /// Project mode: single (default) or workspace
    #[arg(long, value_name = "MODE")]
    mode: Option<String>,

    /// Configuration preset: minimal, api, or fullstack
    #[arg(long, value_name = "PRESET")]
    preset: Option<String>,

    /// Generate GitHub Actions CI/CD workflow
    #[arg(long)]
    ci: bool,

    /// Force overwrite if target directory exists
    #[arg(long)]
    force: bool,

    /// Non-interactive mode (fail if required values missing)
    #[arg(long)]
    non_interactive: bool,

    /// Custom template directory
    #[arg(long, value_name = "DIR")]
    template_dir: Option<PathBuf>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// 创建新项目 / Create a new project
    New {
        /// Project name
        #[arg(value_name = "PROJECT_NAME")]
        project_name: Option<String>,

        #[arg(long)]
        author: Option<String>,

        #[arg(long, value_name = "TYPE")]
        database: Option<String>,

        #[arg(long)]
        auth: bool,

        #[arg(long)]
        biz_error: bool,

        #[arg(long, value_name = "LEVEL")]
        log_level: Option<String>,

        #[arg(long, value_name = "MODE")]
        mode: Option<String>,

        #[arg(long, value_name = "PRESET")]
        preset: Option<String>,

        #[arg(long)]
        ci: bool,

        #[arg(long)]
        force: bool,

        #[arg(long)]
        non_interactive: bool,

        /// Custom template directory
        #[arg(long, value_name = "DIR")]
        template_dir: Option<PathBuf>,
    },
    /// 导出内置模板 / Export built-in templates for customization
    InitTemplate {
        /// Output directory (default: ./templates)
        #[arg(default_value = "./templates")]
        output_dir: PathBuf,

        /// Project mode: single or workspace
        #[arg(long, default_value = "single")]
        mode: String,
    },
    /// 更新已生成的项目 / Update a previously generated project
    Update {
        /// Project directory (default: current directory)
        #[arg(default_value = ".")]
        project_dir: PathBuf,

        /// Show what would change without modifying files
        #[arg(long)]
        dry_run: bool,

        /// Force overwrite all files (ignore user modifications)
        #[arg(long)]
        force: bool,

        /// Custom template directory
        #[arg(long, value_name = "DIR")]
        template_dir: Option<PathBuf>,
    },
}

/// Format error message with troubleshooting guidance
fn format_error_message(error: &CliError) -> String {
    match error {
        CliError::Io(_) | CliError::Git(_) | CliError::Template(_) | CliError::Generation(_) => {
            format!(
                "{}\n\n\
                 🔍 故障排查 / Troubleshooting:\n\
                 1. 检查文件系统权限 / Check file system permissions\n\
                 2. 确保磁盘空间充足 / Ensure sufficient disk space\n\
                 3. 查看日志获取更多信息 / Check logs for more details: RUST_LOG=debug\n\
                 4. 查看帮助 / View help: axum-app-create --help\n\
                 5. 提交bug报告 / Report bug: https://github.com/Yu-Xiao-Sheng/axum-app-create/issues",
                error
            )
        }
        _ => error.to_string(),
    }
}

fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    let cli = Cli::parse();

    println!("\n🦀 axum-app-create CLI Tool v0.3.0");

    // Load user configuration file
    let user_config = UserConfig::load();

    match cli.command {
        Some(Commands::InitTemplate { output_dir, mode }) => run_init_template(&output_dir, &mode),
        Some(Commands::Update {
            project_dir,
            dry_run,
            force,
            template_dir,
        }) => {
            let resolved_dir = resolve_template_dir(template_dir, &user_config);
            run_update(&project_dir, dry_run, force, resolved_dir)
        }
        Some(Commands::New {
            project_name,
            author,
            database,
            auth,
            biz_error,
            log_level,
            mode,
            preset,
            ci,
            force,
            non_interactive,
            template_dir,
        }) => {
            let resolved_dir = resolve_template_dir(template_dir, &user_config);
            run_new(
                project_name,
                author,
                database,
                auth,
                biz_error,
                log_level,
                mode,
                preset,
                ci,
                force,
                non_interactive,
                resolved_dir,
            )
        }
        None => {
            // Backward compatibility: no subcommand = `new`
            let resolved_dir = resolve_template_dir(cli.template_dir, &user_config);
            run_new(
                cli.project_name,
                cli.author,
                cli.database,
                cli.auth,
                cli.biz_error,
                cli.log_level,
                cli.mode,
                cli.preset,
                cli.ci,
                cli.force,
                cli.non_interactive,
                resolved_dir,
            )
        }
    }
}

fn run_init_template(output_dir: &Path, mode: &str) -> anyhow::Result<()> {
    let project_mode = match mode {
        "single" => ProjectMode::Single,
        "workspace" => ProjectMode::Workspace,
        other => {
            eprintln!(
                "\n❌ 无效的模式 / Invalid mode: '{}'\n\
                 💡 有效选项 / Valid options: single, workspace",
                other
            );
            std::process::exit(1);
        }
    };

    match TemplateExporter::export(project_mode, output_dir) {
        Ok(()) => {
            println!(
                "\n💡 使用方法 / Usage:\n\
                 1. 编辑模板文件 / Edit template files in: {}\n\
                 2. 使用自定义模板生成项目 / Generate with custom templates:\n\
                    axum-app-create new my-app --template-dir {}",
                output_dir.display(),
                output_dir.display()
            );
        }
        Err(e) => {
            eprintln!("\n❌ {}", format_error_message(&e));
            std::process::exit(1);
        }
    }

    Ok(())
}

fn run_update(
    project_dir: &Path,
    dry_run: bool,
    force: bool,
    template_dir: Option<PathBuf>,
) -> anyhow::Result<()> {
    if dry_run {
        println!("🔍 Dry-run 模式 / Dry-run mode: 不会修改任何文件 / No files will be modified");
    }

    let engine = UpdateEngine::new(project_dir.to_path_buf(), dry_run, force, template_dir);

    match engine.update(true) {
        Ok(report) => {
            println!("\n{}", report.summary());

            if !report.files_conflicted.is_empty() {
                println!("\n⚠️  冲突文件 / Conflicted files:");
                for f in &report.files_conflicted {
                    println!("  ⚠️  {}", f);
                }
                println!("\n💡 使用 --force 强制覆盖 / Use --force to overwrite all files");
            }
        }
        Err(e) => {
            eprintln!("\n❌ {}", format_error_message(&e));
            std::process::exit(1);
        }
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn run_new(
    project_name: Option<String>,
    author: Option<String>,
    database: Option<String>,
    auth: bool,
    biz_error: bool,
    log_level: Option<String>,
    mode: Option<String>,
    preset: Option<String>,
    ci: bool,
    force: bool,
    non_interactive: bool,
    template_dir: Option<PathBuf>,
) -> anyhow::Result<()> {
    // Check Rust toolchain
    if let Err(e) = check_rust_toolchain() {
        eprintln!("\n❌ {}", e);
        std::process::exit(1);
    }

    // Parse database option from CLI flag
    let cli_database = database.as_deref().map(|d| match d {
        "postgresql" | "postgres" | "pg" => DatabaseOption::PostgreSQL,
        "sqlite" => DatabaseOption::SQLite,
        "both" => DatabaseOption::Both,
        "none" => DatabaseOption::None,
        other => {
            eprintln!(
                "\n❌ Invalid database option: '{}'\n\
                 💡 Valid options: none, postgresql, sqlite, both",
                other
            );
            std::process::exit(1);
        }
    });

    // Parse mode from CLI flag
    let cli_mode = mode.as_deref().map(|m| match m {
        "single" => ProjectMode::Single,
        "workspace" => ProjectMode::Workspace,
        other => {
            eprintln!(
                "\n❌ 无效的模式 / Invalid mode: '{}'\n\
                 💡 有效选项 / Valid options: single, workspace",
                other
            );
            std::process::exit(1);
        }
    });

    // Parse preset from CLI flag
    let cli_preset = preset.as_deref().map(|p| match p {
        "minimal" => Preset::Minimal,
        "api" => Preset::Api,
        "fullstack" => Preset::Fullstack,
        other => {
            eprintln!(
                "\n❌ 无效的预设 / Invalid preset: '{}'\n\
                 💡 有效选项 / Valid options: minimal, api, fullstack",
                other
            );
            std::process::exit(1);
        }
    });

    // Validate log level if provided
    if let Some(ref level) = log_level
        && !["trace", "debug", "info", "warn", "error"].contains(&level.as_str())
    {
        eprintln!(
            "\n❌ Invalid log level: '{}'\n\
                 💡 Valid levels: trace, debug, info, warn, error",
            level
        );
        std::process::exit(1);
    }

    // Determine if we're in interactive mode
    let interactive = !is_non_interactive(non_interactive);

    // Build CLI overrides
    let cli_overrides = axum_app_create::cli::prompts::CliOverrides {
        database: cli_database,
        auth: if auth { Some(true) } else { None },
        biz_error: if biz_error { Some(true) } else { None },
        log_level,
        author,
        mode: cli_mode,
        preset: cli_preset,
        ci: if ci { Some(true) } else { None },
    };

    // Get project configuration
    let config = match prompt_project_config(interactive, project_name, Some(cli_overrides)) {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("\n❌ {}", e);
            std::process::exit(1);
        }
    };

    // Determine project directory
    let project_dir = PathBuf::from(&config.project_name);

    // Generate project (with optional custom templates via TemplateResolver)
    match axum_app_create::generator::project::generate_project_with_templates(
        &project_dir,
        &config,
        interactive,
        force,
        template_dir,
    ) {
        Ok(()) => {
            let message = get_success_message_with_config(&project_dir, &config);
            println!("{}", message);
        }
        Err(e) => {
            eprintln!("\n❌ {}", format_error_message(&e));
            std::process::exit(1);
        }
    }

    Ok(())
}
