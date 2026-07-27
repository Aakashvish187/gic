//! Error types for the DevSecOps & Security Intelligence Engine.

use thiserror::Error;

/// Errors produced by the Security Intelligence Engine.
#[derive(Debug, Error)]
pub enum SecurityError {
    #[error("Secret scanner regex compilation failed: {0}")]
    RegexError(#[from] regex::Error),

    #[error("Policy evaluation error for rule {0}: {1}")]
    PolicyError(String, String),

    #[error("Invalid security rule configuration: {0}")]
    InvalidRule(String),

    #[error("Cache access error: {0}")]
    CacheError(String),

    #[error("Report generation error: {0}")]
    ReportingError(String),

    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Convenience Result type for Security operations.
pub type SecurityResult<T> = Result<T, SecurityError>;
