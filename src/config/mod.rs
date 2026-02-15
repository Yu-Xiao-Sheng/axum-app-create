// Configuration module
//
// This module contains project configuration structures and validation.

use serde::{Deserialize, Serialize};

pub mod user_config;

/// Database option selection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum DatabaseOption {
    /// No database support
    #[default]
    None,
    /// PostgreSQL only (production-focused)
    PostgreSQL,
    /// SQLite only (development-focused)
    SQLite,
    /// Both PostgreSQL and SQLite (environment-based switching)
    Both,
}

impl DatabaseOption {
    /// Returns true if any database support is enabled
    pub fn is_enabled(&self) -> bool {
        matches!(self, Self::PostgreSQL | Self::SQLite | Self::Both)
    }

    /// Returns true if PostgreSQL is supported
    pub fn supports_postgresql(&self) -> bool {
        matches!(self, Self::PostgreSQL | Self::Both)
    }

    /// Returns true if SQLite is supported
    pub fn supports_sqlite(&self) -> bool {
        matches!(self, Self::SQLite | Self::Both)
    }
}

impl std::fmt::Display for DatabaseOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "None"),
            Self::PostgreSQL => write!(f, "PostgreSQL"),
            Self::SQLite => write!(f, "SQLite"),
            Self::Both => write!(f, "PostgreSQL + SQLite"),
        }
    }
}

/// 项目模式 / Project generation mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ProjectMode {
    /// 单包模式（v0.1.1 行为）/ Single-package mode
    #[default]
    Single,
    /// 工作区模式 / Cargo workspace mode
    Workspace,
}

impl std::fmt::Display for ProjectMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Single => write!(f, "single"),
            Self::Workspace => write!(f, "workspace"),
        }
    }
}

/// 配置预设 / Configuration preset
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Preset {
    /// 最小配置 / Minimal - no optional features
    Minimal,
    /// API 开发 / API - PostgreSQL + auth + logging + biz-error
    Api,
    /// 全栈开发 / Fullstack - Both DBs + auth + logging + biz-error
    Fullstack,
}

impl Preset {
    /// 将预设转换为功能集 / Convert preset to FeatureSet
    pub fn to_feature_set(&self) -> FeatureSet {
        match self {
            Self::Minimal => FeatureSet {
                database: DatabaseOption::None,
                authentication: false,
                logging: true,
                biz_error: false,
            },
            Self::Api => FeatureSet {
                database: DatabaseOption::PostgreSQL,
                authentication: true,
                logging: true,
                biz_error: true,
            },
            Self::Fullstack => FeatureSet {
                database: DatabaseOption::Both,
                authentication: true,
                logging: true,
                biz_error: true,
            },
        }
    }
}

impl std::fmt::Display for Preset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Minimal => write!(f, "minimal"),
            Self::Api => write!(f, "api"),
            Self::Fullstack => write!(f, "fullstack"),
        }
    }
}

/// Feature set configuration
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct FeatureSet {
    /// Database support (none, postgresql, sqlite, or both)
    pub database: DatabaseOption,
    /// Authentication support
    pub authentication: bool,
    /// Logging configuration
    pub logging: bool,
    /// Business error handling integration
    pub biz_error: bool,
}

/// Database configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// Database type selected
    pub option: DatabaseOption,
    /// Default connection string format (for .env.example)
    pub default_url: String,
    /// Maximum pool size
    pub max_connections: u32,
    /// Minimum pool size
    pub min_connections: u32,
    /// Enable migration support
    pub migrations: bool,
    /// Migration tool (sqlx-cli recommended)
    pub migration_tool: String,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            option: DatabaseOption::Both,
            default_url: "postgresql://postgres:password@localhost/mydb".to_string(),
            max_connections: 10,
            min_connections: 1,
            migrations: true,
            migration_tool: "sqlx-cli".to_string(),
        }
    }
}

/// Authentication configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AuthConfig {
    /// JWT secret (for .env.example only, not real secret)
    pub example_secret: String,
    /// Token expiration time (in seconds)
    pub expiration_seconds: u64,
    /// Token algorithm (HS256 recommended)
    pub algorithm: String,
    /// Include user model in generated project
    pub include_user_model: bool,
    /// Include login/logout endpoints
    pub include_endpoints: bool,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            example_secret: "your-secret-key-min-32-chars".to_string(),
            expiration_seconds: 24 * 60 * 60, // 24 hours
            algorithm: "HS256".to_string(),
            include_user_model: true,
            include_endpoints: true,
        }
    }
}

/// Logging configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Default log level
    pub default_level: String,
    /// Available log levels
    pub available_levels: Vec<String>,
    /// Log format (json, pretty, compact)
    pub format: String,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            default_level: "info".to_string(),
            available_levels: vec![
                "trace".to_string(),
                "debug".to_string(),
                "info".to_string(),
                "warn".to_string(),
                "error".to_string(),
            ],
            format: "compact".to_string(),
        }
    }
}

/// Business error configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BizErrorConfig {
    /// Include example error codes
    pub include_examples: bool,
    /// Supported languages
    pub languages: Vec<String>,
    /// Path to error definition YAML
    pub error_def_path: String,
    /// Include error handler middleware
    pub include_middleware: bool,
}

impl Default for BizErrorConfig {
    fn default() -> Self {
        Self {
            include_examples: true,
            languages: vec!["en".to_string(), "zh".to_string()],
            error_def_path: "biz_errors.yaml".to_string(),
            include_middleware: true,
        }
    }
}

/// Project configuration for generation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProjectConfig {
    /// Project name (validated)
    pub project_name: String,
    /// Optional features selected
    pub features: FeatureSet,
    /// Author name (optional, from Git config or default)
    pub author_name: Option<String>,
    /// Project description (optional, user-provided or default)
    pub description: Option<String>,
    /// Database configuration (if database feature enabled)
    pub database: Option<DatabaseConfig>,
    /// Authentication configuration (if auth feature enabled)
    pub authentication: Option<AuthConfig>,
    /// Logging configuration (if logging feature enabled)
    pub logging: Option<LoggingConfig>,
    /// Business error handling configuration (if biz-error feature enabled)
    pub biz_error: Option<BizErrorConfig>,
    /// 项目模式 / Project mode (single or workspace)
    pub mode: ProjectMode,
    /// 使用的预设 / Preset used (if any)
    pub preset: Option<Preset>,
    /// 是否生成 CI/CD 配置 / Whether to generate CI/CD config
    pub ci: bool,
}

impl Default for ProjectConfig {
    fn default() -> Self {
        Self {
            project_name: "my-axum-app".to_string(),
            features: FeatureSet::default(),
            author_name: None, // Will try to detect from git
            description: Some("An Axum web application".to_string()),
            database: None,
            authentication: None,
            logging: Some(LoggingConfig::default()),
            biz_error: None,
            mode: ProjectMode::Single,
            preset: None,
            ci: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    // Helper: generate arbitrary Preset values for property testing
    fn arb_preset() -> impl Strategy<Value = Preset> {
        prop_oneof![
            Just(Preset::Minimal),
            Just(Preset::Api),
            Just(Preset::Fullstack),
        ]
    }

    // Property 1: 预设到功能集映射一致性 / Preset-to-FeatureSet idempotency
    // For any Preset, calling to_feature_set() twice returns the same FeatureSet.
    // Validates: Requirements 2.1, 2.2, 2.3
    proptest! {
        #[test]
        fn prop_preset_to_feature_set_idempotent(preset in arb_preset()) {
            let first = preset.to_feature_set();
            let second = preset.to_feature_set();
            prop_assert_eq!(first, second);
        }
    }

    #[test]
    fn test_project_mode_default_is_single() {
        assert_eq!(ProjectMode::default(), ProjectMode::Single);
    }

    #[test]
    fn test_project_mode_display() {
        assert_eq!(ProjectMode::Single.to_string(), "single");
        assert_eq!(ProjectMode::Workspace.to_string(), "workspace");
    }

    #[test]
    fn test_preset_display() {
        assert_eq!(Preset::Minimal.to_string(), "minimal");
        assert_eq!(Preset::Api.to_string(), "api");
        assert_eq!(Preset::Fullstack.to_string(), "fullstack");
    }

    #[test]
    fn test_preset_minimal_feature_set() {
        let fs = Preset::Minimal.to_feature_set();
        assert_eq!(fs.database, DatabaseOption::None);
        assert!(!fs.authentication);
        assert!(fs.logging);
        assert!(!fs.biz_error);
    }

    #[test]
    fn test_preset_api_feature_set() {
        let fs = Preset::Api.to_feature_set();
        assert_eq!(fs.database, DatabaseOption::PostgreSQL);
        assert!(fs.authentication);
        assert!(fs.logging);
        assert!(fs.biz_error);
    }

    #[test]
    fn test_preset_fullstack_feature_set() {
        let fs = Preset::Fullstack.to_feature_set();
        assert_eq!(fs.database, DatabaseOption::Both);
        assert!(fs.authentication);
        assert!(fs.logging);
        assert!(fs.biz_error);
    }

    #[test]
    fn test_project_config_default() {
        let config = ProjectConfig::default();
        assert_eq!(config.mode, ProjectMode::Single);
        assert!(config.preset.is_none());
        assert!(!config.ci);
    }
}
