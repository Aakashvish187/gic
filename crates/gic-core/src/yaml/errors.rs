//! Error types for the YAML Intelligence Engine.

use thiserror::Error;

/// Result type alias for YAML engine operations.
pub type YamlResult<T> = Result<T, YamlError>;

/// Primary error type for YAML parsing, validation, formatting, and schema evaluation.
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum YamlError {
    /// Failed to parse YAML source into AST.
    #[error("YAML Parse Error at L{line}:C{column}: {message}")]
    ParseError {
        message: String,
        line: usize,
        column: usize,
        offset: usize,
    },

    /// Validation failure against generic or schema rules.
    #[error("YAML Validation Error at L{line}:C{column} [{rule_id}]: {message}")]
    ValidationError {
        message: String,
        line: usize,
        column: usize,
        rule_id: String,
    },

    /// Error encountered during YAML formatting.
    #[error("YAML Formatter Error: {message}")]
    FormatError { message: String },

    /// Schema validation or loading error.
    #[error("YAML Schema Error: {message}")]
    SchemaError { message: String },

    /// Cache storage or retrieval failure.
    #[error("YAML Cache Error: {message}")]
    CacheError { message: String },

    /// Invalid indentation level or mixing tabs/spaces.
    #[error("YAML Indentation Error at L{line}: expected {expected} spaces, found {found}")]
    InvalidIndentation {
        message: String,
        line: usize,
        expected: usize,
        found: usize,
    },

    /// Duplicate mapping key found within the same scope.
    #[error("Duplicate YAML key '{key}' at L{line}:C{column} (first defined at L{original_line})")]
    DuplicateKey {
        key: String,
        line: usize,
        column: usize,
        original_line: usize,
    },

    /// Reference to non-existent anchor alias.
    #[error("Unresolved alias '*{alias}' at L{line}:C{column}")]
    UnresolvedAlias {
        alias: String,
        line: usize,
        column: usize,
    },

    /// Circular reference in anchor/alias resolution.
    #[error("Circular alias reference '*{alias}' detected at L{line}:C{column}")]
    CircularAlias {
        alias: String,
        line: usize,
        column: usize,
    },

    /// Unexpected end of file while parsing structure.
    #[error("Unexpected end of YAML input: {message}")]
    UnexpectedEof { message: String },

    /// Generic IO or system error representation.
    #[error("YAML IO Error: {0}")]
    IoError(String),
}
