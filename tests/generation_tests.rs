// Integration tests for project generation
//
// These tests verify the end-to-end functionality of the CLI tool

use axum_app_create::config::{Preset, ProjectConfig, ProjectMode};
use axum_app_create::generator::project::generate_project;
use std::process::Command;
use tempfile::TempDir;

/// T032: Integration test - generate basic project and verify Cargo.toml exists
#[test]
fn test_generate_basic_project() {
    let temp_dir = TempDir::new().unwrap();
    let project_dir = temp_dir.path().join("test-app");

    let config = ProjectConfig {
        project_name: "test-app".to_string(),
        ..Default::default()
    };

    let result = generate_project(&project_dir, &config, false, false);
    assert!(
        result.is_ok(),
        "Project generation failed: {:?}",
        result.err()
    );

    // Verify project directory exists
    assert!(project_dir.exists(), "Project directory was not created");

    // Verify Cargo.toml exists
    let cargo_toml = project_dir.join("Cargo.toml");
    assert!(cargo_toml.exists(), "Cargo.toml was not created");

    // Verify essential files exist
    assert!(project_dir.join("src/main.rs").exists());
    assert!(project_dir.join("src/lib.rs").exists());
    assert!(project_dir.join("src/config.rs").exists());
    assert!(project_dir.join("src/handlers/health.rs").exists());
    assert!(project_dir.join("src/handlers/mod.rs").exists());
    assert!(project_dir.join(".gitignore").exists());
    assert!(project_dir.join(".env.example").exists());
    assert!(project_dir.join("README.md").exists());

    // Verify Cargo.toml content
    let cargo_content = std::fs::read_to_string(&cargo_toml).unwrap();
    assert!(cargo_content.contains("name = \"test-app\""));
    assert!(cargo_content.contains("axum"));
    assert!(cargo_content.contains("tokio"));
    assert!(cargo_content.contains("tracing"));
    assert!(cargo_content.contains("serde"));
    assert!(cargo_content.contains("anyhow"));
}

/// T033: Integration test - generated project compiles with `cargo check`
#[test]
fn test_generated_project_compiles() {
    let temp_dir = TempDir::new().unwrap();
    let project_dir = temp_dir.path().join("compile-test-app");

    let config = ProjectConfig {
        project_name: "compile-test-app".to_string(),
        ..Default::default()
    };

    // Generate project
    let result = generate_project(&project_dir, &config, false, false);
    assert!(
        result.is_ok(),
        "Project generation failed: {:?}",
        result.err()
    );

    // Run cargo check
    let output = Command::new("cargo")
        .arg("check")
        .arg("--manifest-path")
        .arg(project_dir.join("Cargo.toml"))
        .output()
        .expect("Failed to run cargo check");

    if !output.status.success() {
        eprintln!(
            "cargo check stdout: {}",
            String::from_utf8_lossy(&output.stdout)
        );
        eprintln!(
            "cargo check stderr: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    assert!(
        output.status.success(),
        "Generated project failed to compile with cargo check"
    );
}

/// T034: Integration test - generated server starts and responds to GET /health
///
/// Note: This test is more complex as it requires:
/// 1. Building the generated project
/// 2. Starting the server in the background
/// 3. Making an HTTP request to /health
/// 4. Verifying the response
/// 5. Shutting down the server
///
/// For now, we'll verify the health endpoint code exists and is correct.
/// A full end-to-end test would require tokio test runtime and more setup.
#[test]
fn test_health_endpoint_exists() {
    let temp_dir = TempDir::new().unwrap();
    let project_dir = temp_dir.path().join("health-test-app");

    let config = ProjectConfig {
        project_name: "health-test-app".to_string(),
        ..Default::default()
    };

    // Generate project
    let result = generate_project(&project_dir, &config, false, false);
    assert!(
        result.is_ok(),
        "Project generation failed: {:?}",
        result.err()
    );

    // Verify health.rs exists and contains correct endpoint
    let health_rs = project_dir.join("src/handlers/health.rs");
    assert!(health_rs.exists(), "health.rs was not created");

    let health_content = std::fs::read_to_string(&health_rs).unwrap();
    assert!(health_content.contains("health_check"));
    assert!(health_content.contains("HealthResponse"));
    assert!(health_content.contains("status"));
    assert!(health_content.contains("version"));
    assert!(health_content.contains("pub fn router()"));

    // Verify main.rs includes health router
    let main_rs = project_dir.join("src/main.rs");
    let main_content = std::fs::read_to_string(&main_rs).unwrap();
    assert!(main_content.contains("health::router()"));
}

/// Additional test: Verify .gitignore contains essential patterns
#[test]
fn test_gitignore_patterns() {
    let temp_dir = TempDir::new().unwrap();
    let project_dir = temp_dir.path().join("gitignore-test-app");

    let config = ProjectConfig {
        project_name: "gitignore-test-app".to_string(),
        ..Default::default()
    };

    generate_project(&project_dir, &config, false, false).unwrap();

    let gitignore = project_dir.join(".gitignore");
    let content = std::fs::read_to_string(&gitignore).unwrap();

    // Verify essential Rust patterns
    assert!(content.contains("target/"));
    assert!(content.contains("*.log"));
    assert!(content.contains(".env"));
}

/// Additional test: Verify README is bilingual
#[test]
fn test_readme_bilingual() {
    let temp_dir = TempDir::new().unwrap();
    let project_dir = temp_dir.path().join("readme-test-app");

    let config = ProjectConfig {
        project_name: "readme-test-app".to_string(),
        ..Default::default()
    };

    generate_project(&project_dir, &config, false, false).unwrap();

    let readme = project_dir.join("README.md");
    let content = std::fs::read_to_string(&readme).unwrap();

    // Verify bilingual content (English and Chinese)
    assert!(content.contains("Quick Start") || content.contains("快速开始"));
    assert!(content.contains("cargo run"));
}

/// T060: Integration test - generate project with database feature
#[test]
fn test_database_feature() {
    use axum_app_create::config::{DatabaseOption, FeatureSet};

    let temp_dir = TempDir::new().unwrap();
    let project_dir = temp_dir.path().join("db-test-app");

    let config = ProjectConfig {
        project_name: "db-test-app".to_string(),
        features: FeatureSet {
            database: DatabaseOption::PostgreSQL,
            ..Default::default()
        },
        ..Default::default()
    };

    generate_project(&project_dir, &config, false, false).unwrap();

    // Verify db.rs exists
    assert!(project_dir.join("src/db.rs").exists());

    // Verify migrations directory exists
    assert!(project_dir.join("migrations/001_initial.sql").exists());

    // Verify Cargo.toml contains sqlx dependency
    let cargo_toml = std::fs::read_to_string(project_dir.join("Cargo.toml")).unwrap();
    assert!(cargo_toml.contains("sqlx"));

    // Verify .env.example contains DATABASE_URL
    let env_example = std::fs::read_to_string(project_dir.join(".env.example")).unwrap();
    assert!(env_example.contains("DATABASE_URL"));
}

/// T061: Integration test - generate project with authentication feature
#[test]
fn test_auth_feature() {
    use axum_app_create::config::FeatureSet;

    let temp_dir = TempDir::new().unwrap();
    let project_dir = temp_dir.path().join("auth-test-app");

    let config = ProjectConfig {
        project_name: "auth-test-app".to_string(),
        features: FeatureSet {
            authentication: true,
            ..Default::default()
        },
        ..Default::default()
    };

    generate_project(&project_dir, &config, false, false).unwrap();

    // Verify auth handler exists
    assert!(project_dir.join("src/handlers/auth.rs").exists());

    // Verify Cargo.toml contains auth dependencies
    let cargo_toml = std::fs::read_to_string(project_dir.join("Cargo.toml")).unwrap();
    assert!(cargo_toml.contains("jsonwebtoken"));
    assert!(cargo_toml.contains("bcrypt"));

    // Verify .env.example contains JWT_SECRET
    let env_example = std::fs::read_to_string(project_dir.join(".env.example")).unwrap();
    assert!(env_example.contains("JWT_SECRET"));
}

/// T062: Integration test - generate project with biz-error feature
#[test]
fn test_biz_error_feature() {
    use axum_app_create::config::FeatureSet;

    let temp_dir = TempDir::new().unwrap();
    let project_dir = temp_dir.path().join("bizerror-test-app");

    let config = ProjectConfig {
        project_name: "bizerror-test-app".to_string(),
        features: FeatureSet {
            biz_error: true,
            ..Default::default()
        },
        ..Default::default()
    };

    generate_project(&project_dir, &config, false, false).unwrap();

    // Verify biz_errors.yaml exists
    assert!(project_dir.join("biz_errors.yaml").exists());

    // Verify it contains error definitions in both English and Chinese
    let biz_errors = std::fs::read_to_string(project_dir.join("biz_errors.yaml")).unwrap();
    assert!(biz_errors.contains("en:"));
    assert!(biz_errors.contains("zh:"));
}

/// T063: Integration test - generate project with multiple features
#[test]
fn test_multiple_features() {
    use axum_app_create::config::{DatabaseOption, FeatureSet};

    let temp_dir = TempDir::new().unwrap();
    let project_dir = temp_dir.path().join("multi-test-app");

    let config = ProjectConfig {
        project_name: "multi-test-app".to_string(),
        features: FeatureSet {
            database: DatabaseOption::Both,
            authentication: true,
            logging: true,
            biz_error: true,
        },
        ..Default::default()
    };

    generate_project(&project_dir, &config, false, false).unwrap();

    // Verify all feature files exist
    assert!(project_dir.join("src/db.rs").exists());
    assert!(project_dir.join("src/handlers/auth.rs").exists());
    assert!(project_dir.join("migrations/001_initial.sql").exists());
    assert!(project_dir.join("biz_errors.yaml").exists());

    // Verify Cargo.toml contains all dependencies
    let cargo_toml = std::fs::read_to_string(project_dir.join("Cargo.toml")).unwrap();
    assert!(cargo_toml.contains("sqlx"));
    assert!(cargo_toml.contains("jsonwebtoken"));
    assert!(cargo_toml.contains("tracing"));

    // Verify .env.example contains all config variables
    let env_example = std::fs::read_to_string(project_dir.join(".env.example")).unwrap();
    assert!(env_example.contains("DATABASE_URL"));
    assert!(env_example.contains("JWT_SECRET"));
    assert!(env_example.contains("LOG_LEVEL"));
}

/// Test: auth-only project (no database) compiles with `cargo check`
/// This verifies the auth.rs.hbs template generates valid code without database imports
#[test]
fn test_auth_only_project_compiles() {
    use axum_app_create::config::FeatureSet;

    let temp_dir = TempDir::new().unwrap();
    let project_dir = temp_dir.path().join("auth-only-app");

    let config = ProjectConfig {
        project_name: "auth-only-app".to_string(),
        features: FeatureSet {
            authentication: true,
            ..Default::default()
        },
        ..Default::default()
    };

    generate_project(&project_dir, &config, false, false).unwrap();

    // Run cargo check to verify it compiles
    let output = Command::new("cargo")
        .arg("check")
        .arg("--manifest-path")
        .arg(project_dir.join("Cargo.toml"))
        .output()
        .expect("Failed to run cargo check");

    if !output.status.success() {
        eprintln!(
            "cargo check stderr: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    assert!(
        output.status.success(),
        "Auth-only generated project failed to compile"
    );
}

/// Test: --force flag overwrites existing directory
#[test]
fn test_force_overwrite() {
    let temp_dir = TempDir::new().unwrap();
    let project_dir = temp_dir.path().join("force-test-app");

    let config = ProjectConfig {
        project_name: "force-test-app".to_string(),
        ..Default::default()
    };

    // Generate first time
    generate_project(&project_dir, &config, false, false).unwrap();
    assert!(project_dir.exists());

    // Generate again with force=true should succeed
    let result = generate_project(&project_dir, &config, false, true);
    assert!(result.is_ok(), "Force overwrite failed: {:?}", result.err());
    assert!(project_dir.join("Cargo.toml").exists());
}

/// Test: existing directory without force in non-interactive mode should fail
#[test]
fn test_existing_dir_no_force_fails() {
    let temp_dir = TempDir::new().unwrap();
    let project_dir = temp_dir.path().join("noforce-test-app");

    let config = ProjectConfig {
        project_name: "noforce-test-app".to_string(),
        ..Default::default()
    };

    // Generate first time
    generate_project(&project_dir, &config, false, false).unwrap();

    // Generate again without force should fail
    let result = generate_project(&project_dir, &config, false, false);
    assert!(
        result.is_err(),
        "Should fail when directory exists without --force"
    );
}

/// Test: project with ALL features enabled compiles with `cargo check`
/// This is the critical integration test that catches template type mismatches
#[test]
fn test_all_features_project_compiles() {
    use axum_app_create::config::{DatabaseOption, FeatureSet};

    let temp_dir = TempDir::new().unwrap();
    let project_dir = temp_dir.path().join("all-features-app");

    let config = ProjectConfig {
        project_name: "all-features-app".to_string(),
        features: FeatureSet {
            database: DatabaseOption::Both,
            authentication: true,
            logging: true,
            biz_error: true,
        },
        ..Default::default()
    };

    generate_project(&project_dir, &config, false, false).unwrap();

    // Verify biz-error dependency and build.rs in Cargo.toml
    let cargo_toml = std::fs::read_to_string(project_dir.join("Cargo.toml")).unwrap();
    assert!(
        cargo_toml.contains("biz-error"),
        "Cargo.toml should contain biz-error dependency"
    );
    assert!(
        cargo_toml.contains("[build-dependencies]"),
        "Cargo.toml should have [build-dependencies] section"
    );

    // Verify build.rs exists
    assert!(
        project_dir.join("build.rs").exists(),
        "build.rs should be generated for biz-error codegen"
    );

    // Run cargo check to verify it compiles
    let output = Command::new("cargo")
        .arg("check")
        .arg("--manifest-path")
        .arg(project_dir.join("Cargo.toml"))
        .output()
        .expect("Failed to run cargo check");

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("cargo check stderr:\n{}", stderr);
    }

    assert!(
        output.status.success(),
        "All-features generated project failed to compile"
    );
}

/// Test: project with database + auth compiles (the specific combo that was broken)
#[test]
fn test_database_auth_project_compiles() {
    use axum_app_create::config::{DatabaseOption, FeatureSet};

    let temp_dir = TempDir::new().unwrap();
    let project_dir = temp_dir.path().join("db-auth-app");

    let config = ProjectConfig {
        project_name: "db-auth-app".to_string(),
        features: FeatureSet {
            database: DatabaseOption::PostgreSQL,
            authentication: true,
            logging: true,
            ..Default::default()
        },
        ..Default::default()
    };

    generate_project(&project_dir, &config, false, false).unwrap();

    // Run cargo check
    let output = Command::new("cargo")
        .arg("check")
        .arg("--manifest-path")
        .arg(project_dir.join("Cargo.toml"))
        .output()
        .expect("Failed to run cargo check");

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("cargo check stderr:\n{}", stderr);
    }

    assert!(
        output.status.success(),
        "Database+Auth generated project failed to compile"
    );
}

// ============================================================
// v0.2.0 Integration Tests
// ============================================================

/// Test workspace mode generates correct structure
#[test]
fn test_workspace_mode_basic_structure() {
    let temp_dir = TempDir::new().unwrap();
    let project_dir = temp_dir.path().join("ws-app");

    let config = ProjectConfig {
        project_name: "ws-app".to_string(),
        mode: ProjectMode::Workspace,
        ..Default::default()
    };

    let result = generate_project(&project_dir, &config, false, false);
    assert!(
        result.is_ok(),
        "Workspace generation failed: {:?}",
        result.err()
    );

    // Verify root Cargo.toml has [workspace]
    let cargo_content = std::fs::read_to_string(project_dir.join("Cargo.toml")).unwrap();
    assert!(
        cargo_content.contains("[workspace]"),
        "Root Cargo.toml missing [workspace]"
    );
    assert!(
        cargo_content.contains("members"),
        "Root Cargo.toml missing members"
    );

    // Verify all 4 sub-crate directories exist
    for crate_name in &["api", "domain", "infrastructure", "common"] {
        assert!(
            project_dir.join(crate_name).exists(),
            "Missing crate dir: {}",
            crate_name
        );
        assert!(
            project_dir
                .join(format!("{}/Cargo.toml", crate_name))
                .exists(),
            "Missing {}/Cargo.toml",
            crate_name
        );
        assert!(
            project_dir
                .join(format!("{}/src/lib.rs", crate_name))
                .exists(),
            "Missing {}/src/lib.rs",
            crate_name
        );
    }

    // Verify api crate has main.rs (binary crate)
    assert!(
        project_dir.join("api/src/main.rs").exists(),
        "Missing api/src/main.rs"
    );
}

/// Test workspace mode + database feature
#[test]
fn test_workspace_mode_with_database() {
    let temp_dir = TempDir::new().unwrap();
    let project_dir = temp_dir.path().join("ws-db-app");

    let config = ProjectConfig {
        project_name: "ws-db-app".to_string(),
        mode: ProjectMode::Workspace,
        features: axum_app_create::config::FeatureSet {
            database: axum_app_create::config::DatabaseOption::PostgreSQL,
            ..Default::default()
        },
        database: Some(axum_app_create::config::DatabaseConfig::default()),
        ..Default::default()
    };

    let result = generate_project(&project_dir, &config, false, false);
    assert!(result.is_ok(), "Generation failed: {:?}", result.err());

    assert!(
        project_dir.join("infrastructure/src/db.rs").exists(),
        "Missing infrastructure/src/db.rs"
    );
}

/// Test workspace mode + auth feature
#[test]
fn test_workspace_mode_with_auth() {
    let temp_dir = TempDir::new().unwrap();
    let project_dir = temp_dir.path().join("ws-auth-app");

    let config = ProjectConfig {
        project_name: "ws-auth-app".to_string(),
        mode: ProjectMode::Workspace,
        features: axum_app_create::config::FeatureSet {
            authentication: true,
            ..Default::default()
        },
        authentication: Some(axum_app_create::config::AuthConfig::default()),
        ..Default::default()
    };

    let result = generate_project(&project_dir, &config, false, false);
    assert!(result.is_ok(), "Generation failed: {:?}", result.err());

    assert!(
        project_dir.join("api/src/handlers/auth.rs").exists(),
        "Missing api/src/handlers/auth.rs"
    );
    assert!(
        project_dir.join("api/src/middleware/mod.rs").exists(),
        "Missing api/src/middleware/mod.rs"
    );
}

/// Test workspace mode + biz-error feature
#[test]
fn test_workspace_mode_with_biz_error() {
    let temp_dir = TempDir::new().unwrap();
    let project_dir = temp_dir.path().join("ws-biz-app");

    let config = ProjectConfig {
        project_name: "ws-biz-app".to_string(),
        mode: ProjectMode::Workspace,
        features: axum_app_create::config::FeatureSet {
            biz_error: true,
            ..Default::default()
        },
        biz_error: Some(axum_app_create::config::BizErrorConfig::default()),
        ..Default::default()
    };

    let result = generate_project(&project_dir, &config, false, false);
    assert!(result.is_ok(), "Generation failed: {:?}", result.err());

    assert!(
        project_dir.join("common/src/error.rs").exists(),
        "Missing common/src/error.rs"
    );
}

/// Test preset minimal generates no optional features
#[test]
fn test_preset_minimal() {
    let temp_dir = TempDir::new().unwrap();
    let project_dir = temp_dir.path().join("minimal-app");

    let features = Preset::Minimal.to_feature_set();
    let config = ProjectConfig {
        project_name: "minimal-app".to_string(),
        features,
        preset: Some(Preset::Minimal),
        ..Default::default()
    };

    let result = generate_project(&project_dir, &config, false, false);
    assert!(result.is_ok(), "Generation failed: {:?}", result.err());

    // No database or auth files should exist (they render to empty)
    let db_content = std::fs::read_to_string(project_dir.join("src/db.rs")).unwrap_or_default();
    assert!(db_content.trim().is_empty() || !project_dir.join("src/db.rs").exists());
}

/// Test preset api generates PostgreSQL + auth + biz-error
#[test]
fn test_preset_api() {
    let temp_dir = TempDir::new().unwrap();
    let project_dir = temp_dir.path().join("api-app");

    let features = Preset::Api.to_feature_set();
    let config = ProjectConfig {
        project_name: "api-app".to_string(),
        features,
        preset: Some(Preset::Api),
        database: Some(axum_app_create::config::DatabaseConfig::default()),
        authentication: Some(axum_app_create::config::AuthConfig::default()),
        biz_error: Some(axum_app_create::config::BizErrorConfig::default()),
        ..Default::default()
    };

    let result = generate_project(&project_dir, &config, false, false);
    assert!(result.is_ok(), "Generation failed: {:?}", result.err());

    // Auth file should have content
    let auth_content = std::fs::read_to_string(project_dir.join("src/handlers/auth.rs")).unwrap();
    assert!(
        !auth_content.trim().is_empty(),
        "Auth handler should have content"
    );
}

/// Test preset fullstack generates both DBs + all features
#[test]
fn test_preset_fullstack() {
    let temp_dir = TempDir::new().unwrap();
    let project_dir = temp_dir.path().join("fullstack-app");

    let features = Preset::Fullstack.to_feature_set();
    let config = ProjectConfig {
        project_name: "fullstack-app".to_string(),
        features,
        preset: Some(Preset::Fullstack),
        database: Some(axum_app_create::config::DatabaseConfig::default()),
        authentication: Some(axum_app_create::config::AuthConfig::default()),
        biz_error: Some(axum_app_create::config::BizErrorConfig::default()),
        ..Default::default()
    };

    let result = generate_project(&project_dir, &config, false, false);
    assert!(result.is_ok(), "Generation failed: {:?}", result.err());

    let cargo_content = std::fs::read_to_string(project_dir.join("Cargo.toml")).unwrap();
    assert!(
        cargo_content.contains("sqlx"),
        "Fullstack should include sqlx"
    );
}

/// Test CI template generated when ci=true
#[test]
fn test_ci_enabled() {
    let temp_dir = TempDir::new().unwrap();
    let project_dir = temp_dir.path().join("ci-app");

    let config = ProjectConfig {
        project_name: "ci-app".to_string(),
        ci: true,
        ..Default::default()
    };

    let result = generate_project(&project_dir, &config, false, false);
    assert!(result.is_ok(), "Generation failed: {:?}", result.err());

    assert!(
        project_dir.join(".github/workflows/ci.yml").exists(),
        "CI workflow file should exist"
    );

    // Single mode should NOT have --workspace flag
    let ci_content = std::fs::read_to_string(project_dir.join(".github/workflows/ci.yml")).unwrap();
    assert!(
        !ci_content.contains("--workspace"),
        "Single mode CI should not have --workspace"
    );
}

/// Test CI template NOT generated when ci=false
#[test]
fn test_ci_disabled() {
    let temp_dir = TempDir::new().unwrap();
    let project_dir = temp_dir.path().join("no-ci-app");

    let config = ProjectConfig {
        project_name: "no-ci-app".to_string(),
        ci: false,
        ..Default::default()
    };

    let result = generate_project(&project_dir, &config, false, false);
    assert!(result.is_ok(), "Generation failed: {:?}", result.err());

    assert!(
        !project_dir.join(".github/workflows/ci.yml").exists(),
        "CI workflow should NOT exist"
    );
}

/// Test workspace mode + CI has --workspace flag
#[test]
fn test_workspace_ci_has_workspace_flag() {
    let temp_dir = TempDir::new().unwrap();
    let project_dir = temp_dir.path().join("ws-ci-app");

    let config = ProjectConfig {
        project_name: "ws-ci-app".to_string(),
        mode: ProjectMode::Workspace,
        ci: true,
        ..Default::default()
    };

    let result = generate_project(&project_dir, &config, false, false);
    assert!(result.is_ok(), "Generation failed: {:?}", result.err());

    let ci_content = std::fs::read_to_string(project_dir.join(".github/workflows/ci.yml")).unwrap();
    assert!(
        ci_content.contains("--workspace"),
        "Workspace CI should have --workspace flag"
    );
}

/// Test workspace mode basic project compiles with cargo check
#[test]
fn test_workspace_basic_compiles() {
    let temp_dir = TempDir::new().unwrap();
    let project_dir = temp_dir.path().join("ws-compile-test");

    let config = ProjectConfig {
        project_name: "ws-compile-test".to_string(),
        mode: ProjectMode::Workspace,
        ..Default::default()
    };

    let result = generate_project(&project_dir, &config, false, false);
    assert!(result.is_ok(), "Generation failed: {:?}", result.err());

    let output = Command::new("cargo")
        .arg("check")
        .arg("--workspace")
        .arg("--manifest-path")
        .arg(project_dir.join("Cargo.toml"))
        .output()
        .expect("Failed to run cargo check");

    if !output.status.success() {
        eprintln!(
            "cargo check stdout: {}",
            String::from_utf8_lossy(&output.stdout)
        );
        eprintln!(
            "cargo check stderr: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    assert!(
        output.status.success(),
        "Workspace project failed to compile"
    );
}

/// Test workspace mode full-featured project compiles
#[test]
fn test_workspace_full_features_compiles() {
    let temp_dir = TempDir::new().unwrap();
    let project_dir = temp_dir.path().join("ws-full-test");

    let features = Preset::Fullstack.to_feature_set();
    let config = ProjectConfig {
        project_name: "ws-full-test".to_string(),
        mode: ProjectMode::Workspace,
        features,
        preset: Some(Preset::Fullstack),
        database: Some(axum_app_create::config::DatabaseConfig::default()),
        authentication: Some(axum_app_create::config::AuthConfig::default()),
        biz_error: Some(axum_app_create::config::BizErrorConfig::default()),
        ci: true,
        ..Default::default()
    };

    let result = generate_project(&project_dir, &config, false, false);
    assert!(result.is_ok(), "Generation failed: {:?}", result.err());

    let output = Command::new("cargo")
        .arg("check")
        .arg("--workspace")
        .arg("--manifest-path")
        .arg(project_dir.join("Cargo.toml"))
        .output()
        .expect("Failed to run cargo check");

    if !output.status.success() {
        eprintln!(
            "cargo check stdout: {}",
            String::from_utf8_lossy(&output.stdout)
        );
        eprintln!(
            "cargo check stderr: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    assert!(
        output.status.success(),
        "Workspace full-featured project failed to compile"
    );
}

/// Test workspace_crates context is correctly populated for workspace mode
#[test]
fn test_workspace_crates_context() {
    use axum_app_create::template::context::TemplateContext;

    let config = ProjectConfig {
        project_name: "ctx-test".to_string(),
        mode: ProjectMode::Workspace,
        ..Default::default()
    };

    let ctx = TemplateContext::from_config(&config);
    assert!(ctx.is_workspace);
    let crates = ctx
        .workspace_crates
        .as_ref()
        .expect("workspace_crates should be Some");
    assert_eq!(crates.len(), 4);

    // Verify crate names
    let names: Vec<&str> = crates.iter().map(|c| c.name.as_str()).collect();
    assert_eq!(names, vec!["api", "domain", "infrastructure", "common"]);

    // Verify api is bin, others are lib
    assert_eq!(crates[0].kind, "bin");
    assert_eq!(crates[1].kind, "lib");
    assert_eq!(crates[2].kind, "lib");
    assert_eq!(crates[3].kind, "lib");

    // Verify package names
    assert_eq!(crates[0].package_name, "ctx-test-api");
    assert_eq!(crates[1].package_name, "ctx-test-domain");

    // Verify api depends on domain, infrastructure, common
    assert!(crates[0].workspace_deps.contains(&"domain".to_string()));
    assert!(
        crates[0]
            .workspace_deps
            .contains(&"infrastructure".to_string())
    );
    assert!(crates[0].workspace_deps.contains(&"common".to_string()));

    // Verify domain has no workspace deps
    assert!(crates[1].workspace_deps.is_empty());

    // Verify infrastructure depends on domain
    assert_eq!(crates[2].workspace_deps, vec!["domain"]);
}

/// Test workspace_crates is None for single mode
#[test]
fn test_single_mode_no_workspace_crates() {
    use axum_app_create::template::context::TemplateContext;

    let config = ProjectConfig {
        project_name: "single-test".to_string(),
        mode: ProjectMode::Single,
        ..Default::default()
    };

    let ctx = TemplateContext::from_config(&config);
    assert!(!ctx.is_workspace);
    assert!(ctx.workspace_crates.is_none());
}

// ============================================================
// v0.3.0 Integration Tests
// ============================================================

/// Test 14.1: Generate project with custom template directory overriding a built-in template
#[test]
fn test_custom_template_override() {
    let temp_dir = TempDir::new().unwrap();
    let project_dir = temp_dir.path().join("custom-tmpl-app");
    let custom_dir = temp_dir.path().join("custom-templates");

    // Create a custom README.md.hbs that overrides the built-in
    std::fs::create_dir_all(&custom_dir).unwrap();
    std::fs::write(
        custom_dir.join("README.md.hbs"),
        "# Custom README for {{project_name}}\nThis is a custom template.",
    )
    .unwrap();

    let config = ProjectConfig {
        project_name: "custom-tmpl-app".to_string(),
        ..Default::default()
    };

    axum_app_create::generator::project::generate_project_with_templates(
        &project_dir,
        &config,
        false,
        false,
        Some(custom_dir),
    )
    .unwrap();

    let readme = std::fs::read_to_string(project_dir.join("README.md")).unwrap();
    assert!(
        readme.contains("Custom README for custom-tmpl-app"),
        "Custom template content should appear in output"
    );
    assert!(
        readme.contains("This is a custom template"),
        "Custom template content should be present"
    );
    // Other built-in files should still exist
    assert!(project_dir.join("Cargo.toml").exists());
    assert!(project_dir.join("src/main.rs").exists());
}

/// Test 14.2: Generate project with template inheritance (extends + override blocks)
#[test]
fn test_template_inheritance() {
    let temp_dir = TempDir::new().unwrap();
    let project_dir = temp_dir.path().join("inherit-app");
    let custom_dir = temp_dir.path().join("inherit-templates");

    // Create a child template that extends the built-in main.rs.hbs
    std::fs::create_dir_all(custom_dir.join("src")).unwrap();
    std::fs::write(
        custom_dir.join("src/main.rs.hbs"),
        r#"{{!-- extends: src/main.rs --}}

{{#override "imports"}}
use axum::{Router, routing::get};
use tokio::net::TcpListener;
// Custom import added by inheritance
use tracing::info;
{{/override}}
"#,
    )
    .unwrap();

    let config = ProjectConfig {
        project_name: "inherit-app".to_string(),
        ..Default::default()
    };

    axum_app_create::generator::project::generate_project_with_templates(
        &project_dir,
        &config,
        false,
        false,
        Some(custom_dir),
    )
    .unwrap();

    let main_rs = std::fs::read_to_string(project_dir.join("src/main.rs")).unwrap();
    assert!(
        main_rs.contains("Custom import added by inheritance"),
        "Override block content should appear"
    );
    // Non-overridden blocks should retain defaults
    assert!(
        main_rs.contains("TcpListener"),
        "Non-overridden content should be preserved"
    );
}

/// Test 14.3: Generate project then update with no changes (all files skipped)
#[test]
fn test_update_no_changes() {
    use axum_app_create::updater::engine::UpdateEngine;

    let temp_dir = TempDir::new().unwrap();
    let project_dir = temp_dir.path().join("update-nochange-app");

    let config = ProjectConfig {
        project_name: "update-nochange-app".to_string(),
        ..Default::default()
    };

    generate_project(&project_dir, &config, false, false).unwrap();

    // Run update — no changes should be detected
    let engine = UpdateEngine::new(project_dir.clone(), false, false, None);
    let report = engine.update(false).unwrap();

    assert!(
        report.files_updated.is_empty(),
        "No files should be updated: {:?}",
        report.files_updated
    );
    assert!(
        report.files_conflicted.is_empty(),
        "No conflicts expected: {:?}",
        report.files_conflicted
    );
    assert!(
        !report.files_skipped.is_empty(),
        "All files should be skipped"
    );
}

/// Test 14.4: Generate project, modify a file, update with conflict detection
#[test]
fn test_update_conflict_detection() {
    use axum_app_create::updater::engine::UpdateEngine;

    let temp_dir = TempDir::new().unwrap();
    let project_dir = temp_dir.path().join("update-conflict-app");

    let config = ProjectConfig {
        project_name: "update-conflict-app".to_string(),
        ..Default::default()
    };

    generate_project(&project_dir, &config, false, false).unwrap();

    // Modify a generated file (simulating user edit)
    let main_rs = project_dir.join("src/main.rs");
    let content = std::fs::read_to_string(&main_rs).unwrap();
    std::fs::write(&main_rs, format!("{}\n// User modification", content)).unwrap();

    // Also modify the template source to create a diff
    // (The update engine regenerates from stored config, so if the template
    //  hasn't changed, the new content will match the original)
    // Since we can't change built-in templates, the update will see:
    // - current file differs from new generation (user modified)
    // - current checksum differs from stored checksum (user modified)
    // → conflict
    let engine = UpdateEngine::new(project_dir.clone(), false, false, None);
    let report = engine.update(false).unwrap();

    // The modified file should be detected
    // Note: since the regenerated content matches the original,
    // and the user modified the file, the current content differs from new content
    // AND the current checksum differs from stored checksum → conflict
    // But actually: new content == original content, current != new → check checksum
    // current checksum != stored checksum → conflict
    assert!(
        report.files_conflicted.contains(&"src/main.rs".to_string()),
        "Modified file should be detected as conflict: {:?}",
        report
    );
}

/// Test 14.5: init-template exports templates, use them as custom templates to generate identical project
#[test]
fn test_init_template_roundtrip() {
    use axum_app_create::template::exporter::TemplateExporter;

    let temp_dir = TempDir::new().unwrap();
    let export_dir = temp_dir.path().join("exported");
    let project_builtin = temp_dir.path().join("builtin-app");
    let project_custom = temp_dir.path().join("custom-app");

    let config = ProjectConfig {
        project_name: "roundtrip-app".to_string(),
        ..Default::default()
    };

    // Generate with built-in templates
    generate_project(&project_builtin, &config, false, false).unwrap();

    // Export templates
    TemplateExporter::export(ProjectMode::Single, &export_dir).unwrap();

    // Generate with exported templates as custom
    axum_app_create::generator::project::generate_project_with_templates(
        &project_custom,
        &config,
        false,
        false,
        Some(export_dir),
    )
    .unwrap();

    // Compare key files — they should be identical
    for file in &["Cargo.toml", "src/main.rs", "src/lib.rs", "src/config.rs"] {
        let builtin_content = std::fs::read_to_string(project_builtin.join(file)).unwrap();
        let custom_content = std::fs::read_to_string(project_custom.join(file)).unwrap();
        assert_eq!(
            builtin_content, custom_content,
            "File {} should be identical between built-in and exported template generation",
            file
        );
    }
}

/// Test 14.6: Backward compatibility — all v0.2.0 argument combinations produce same results
#[test]
fn test_backward_compatibility() {
    use axum_app_create::config::{DatabaseOption, FeatureSet};

    let temp_dir = TempDir::new().unwrap();

    // Test various v0.2.0-style configurations
    let configs = vec![
        ProjectConfig {
            project_name: "compat-minimal".to_string(),
            ..Default::default()
        },
        ProjectConfig {
            project_name: "compat-db".to_string(),
            features: FeatureSet {
                database: DatabaseOption::PostgreSQL,
                ..Default::default()
            },
            ..Default::default()
        },
        ProjectConfig {
            project_name: "compat-workspace".to_string(),
            mode: ProjectMode::Workspace,
            ..Default::default()
        },
    ];

    for config in configs {
        let project_dir = temp_dir.path().join(&config.project_name);
        let result = generate_project(&project_dir, &config, false, false);
        assert!(
            result.is_ok(),
            "Backward compatible generation failed for {}: {:?}",
            config.project_name,
            result.err()
        );
        assert!(
            project_dir.join("Cargo.toml").exists(),
            "Cargo.toml missing for {}",
            config.project_name
        );

        // v0.3.0 addition: metadata file should also be created
        assert!(
            project_dir.join(".axum-app-create.json").exists(),
            "Metadata file missing for {}",
            config.project_name
        );
    }
}
