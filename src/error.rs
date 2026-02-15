// Error types for the CLI tool
//
// This module defines all error types using thiserror.

use thiserror::Error;

/// Main error type for the CLI tool
#[derive(Error, Debug)]
pub enum CliError {
    #[error("Invalid project name '{0}': {1}")]
    InvalidName(String, String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Template error: {0}")]
    Template(String),

    #[error("Generation error: {0}")]
    Generation(String),

    #[error("Git error: {0}")]
    Git(String),

    #[error("Config error: {0}")]
    Config(String),

    #[error("Toolchain error: {0}")]
    ToolchainError(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Prompt error: {0}")]
    PromptError(#[from] inquire::InquireError),
}

/// Result type alias for CLI operations
pub type Result<T> = std::result::Result<T, CliError>;
