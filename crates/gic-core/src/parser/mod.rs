//! Universal Language Parsing Engine module for GIC (General Infrastructure Console).
//!
//! Provides production-grade, memory-safe parsing of infrastructure configuration
//! files and programming languages into structured syntax trees, token streams, and diagnostics.

#![forbid(unsafe_code)]

pub mod cache;
pub mod diagnostics;
pub mod engine;
pub mod errors;
pub mod language;
pub mod loader;
pub mod node;
pub mod parser_trait;
pub mod parsers;
pub mod position;
pub mod registry;
pub mod token;
pub mod tree;
pub mod utils;

#[cfg(test)]
pub mod tests;

// Re-export primary types for convenient consumer access
pub use cache::{CacheMetrics, ParseCache};
pub use diagnostics::{Diagnostic, DiagnosticSeverity};
pub use engine::ParsingEngine;
pub use errors::ParseError;
pub use language::{LanguageDetector, LanguageId, LanguageSpec};
pub use loader::ParserLoader;
pub use node::{NodeKind, SyntaxNode};
pub use parser_trait::{LanguageParser, TreeSitterBackend};
pub use position::{Position, TextChange, TextRange};
pub use registry::ParserRegistry;
pub use token::{Token, TokenKind, TokenStream};
pub use tree::{SymbolInformation, SyntaxTree};
pub use utils::{byte_offset_to_position, compute_line_offsets, hash_source};
