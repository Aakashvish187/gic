//! Trait definitions for language parsers and backend integrations.

use crate::parser::errors::ParseError;
use crate::parser::language::LanguageId;
use crate::parser::position::TextChange;
use crate::parser::tree::SyntaxTree;
use std::fmt::Debug;

/// Core interface implemented by all language parsers.
pub trait LanguageParser: Send + Sync + Debug {
    /// Returns the language identifier handled by this parser.
    fn language(&self) -> LanguageId;

    /// Parses full source text into a structured `SyntaxTree`.
    fn parse(&self, source: &str, old_tree: Option<&SyntaxTree>) -> Result<SyntaxTree, ParseError>;

    /// Incrementally updates a syntax tree given a text change delta.
    fn parse_incremental(
        &self,
        source: &str,
        change: &TextChange,
        old_tree: &SyntaxTree,
    ) -> Result<SyntaxTree, ParseError> {
        // Default implementation re-parses source text with old tree context
        let _ = change;
        self.parse(source, Some(old_tree))
    }

    /// Indicates whether this parser uses a Tree-sitter C grammar backend.
    fn supports_tree_sitter(&self) -> bool {
        false
    }
}

/// Bridge trait for Tree-sitter backend integration adapters.
pub trait TreeSitterBackend: Send + Sync + Debug {
    /// Returns the tree-sitter language handle or parser identifier.
    fn backend_name(&self) -> &'static str;

    /// Parses source using native Tree-sitter C bindings.
    fn parse_tree_sitter(&self, source: &str) -> Result<SyntaxTree, ParseError>;
}
