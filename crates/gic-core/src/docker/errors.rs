//! Error types and Result alias for Docker & Docker Compose Intelligence Engine.

use thiserror::Error;

use crate::yaml::parser::Span;

/// Error variants encountered during Dockerfile or Compose parsing and analysis.
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum DockerError {
    /// Dockerfile syntax error at line/column location.
    #[error("Dockerfile syntax error at line {line}: {message}")]
    SyntaxError {
        /// Line number (1-indexed).
        line: usize,
        /// Detailed error message.
        message: String,
        /// Span location in source code.
        span: Span,
    },

    /// Invalid or unrecognized instruction.
    #[error("Unknown Dockerfile instruction '{instruction}' at line {line}")]
    UnknownInstruction {
        /// Instruction name.
        instruction: String,
        /// Line number (1-indexed).
        line: usize,
        /// Span location.
        span: Span,
    },

    /// Invalid Docker Compose document structure.
    #[error("Docker Compose document error: {message}")]
    ComposeError {
        /// Detailed error message.
        message: String,
        /// Span location.
        span: Span,
    },

    /// Cache operation error.
    #[error("Docker cache operation error: {0}")]
    CacheError(String),
}

/// Specialized Result type for Docker engine operations.
pub type DockerResult<T> = Result<T, DockerError>;
