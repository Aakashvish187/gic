//! Error types and Result alias for Terraform Intelligence Engine.

use thiserror::Error;

use crate::yaml::parser::Span;

/// Error variants encountered during Terraform (HCL) parsing and analysis.
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum TerraformError {
    /// HCL syntax error at line/column location.
    #[error("Terraform HCL syntax error at line {line}: {message}")]
    SyntaxError {
        /// Line number (1-indexed).
        line: usize,
        /// Detailed error message.
        message: String,
        /// Span location in source code.
        span: Span,
    },

    /// Invalid or unrecognized block structure.
    #[error("Unknown or invalid block type '{block_type}' at line {line}")]
    InvalidBlock {
        /// Block type identifier.
        block_type: String,
        /// Line number (1-indexed).
        line: usize,
        /// Span location.
        span: Span,
    },

    /// Unresolved variable or reference interpolation.
    #[error("Unresolved reference expression '{expression}' at line {line}")]
    UnresolvedReference {
        /// Expression text.
        expression: String,
        /// Line number (1-indexed).
        line: usize,
        /// Span location.
        span: Span,
    },

    /// Cache operation error.
    #[error("Terraform cache operation error: {0}")]
    CacheError(String),
}

/// Specialized Result type for Terraform engine operations.
pub type TerraformResult<T> = Result<T, TerraformError>;
