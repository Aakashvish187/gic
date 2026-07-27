//! Error types for the language parsing engine.

use crate::parser::language::LanguageId;
use thiserror::Error;

/// Errors that can occur during language detection, parser registration, or parsing.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ParseError {
    /// Specified language is not supported or recognized.
    #[error("Language not supported: {0}")]
    LanguageNotSupported(String),

    /// No parser is registered for the specified language.
    #[error("No parser found for language: {0}")]
    ParserNotFound(LanguageId),

    /// Parsing failed due to an unrecoverable structural error.
    #[error("Parsing failed: {0}")]
    ParsingFailed(String),

    /// Cache lookup or store failure.
    #[error("Parse cache error: {0}")]
    CacheError(String),

    /// I/O error occurred during source reading or loading.
    #[error("I/O error: {0}")]
    IoError(String),

    /// Invalid text range specified for parsing or incremental update.
    #[error("Invalid text range: {0}")]
    InvalidRange(String),

    /// Error from Tree-sitter backend integration.
    #[error("Tree-sitter error: {0}")]
    TreeSitterError(String),
}
