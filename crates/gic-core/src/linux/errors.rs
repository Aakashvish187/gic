//! Error types and Result alias for Linux & Bash Intelligence Engine.

use thiserror::Error;

use crate::yaml::parser::Span;

/// Error variants encountered during Bash script or Linux configuration parsing and analysis.
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum LinuxError {
    /// Shell syntax or command structure error.
    #[error("Shell syntax error at line {line}: {message}")]
    SyntaxError {
        /// Line number (1-indexed).
        line: usize,
        /// Detailed error message.
        message: String,
        /// Span location in source code.
        span: Span,
    },

    /// Unrecognized or invalid shell command.
    #[error("Unknown or invalid command '{command}' at line {line}")]
    UnknownCommand {
        /// Command name.
        command: String,
        /// Line number (1-indexed).
        line: usize,
        /// Span location.
        span: Span,
    },

    /// Invalid configuration file structure (e.g. systemd or sshd_config).
    #[error("Linux configuration error at line {line}: {message}")]
    ConfigError {
        /// Detailed error message.
        message: String,
        /// Line number.
        line: usize,
        /// Span location.
        span: Span,
    },

    /// Cache operation error.
    #[error("Linux cache operation error: {0}")]
    CacheError(String),
}

/// Specialized Result type for Linux engine operations.
pub type LinuxResult<T> = Result<T, LinuxError>;
