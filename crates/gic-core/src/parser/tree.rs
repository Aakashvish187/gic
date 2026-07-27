//! Abstract Syntax Tree container encapsulating root nodes, tokens, diagnostics, and symbol queries.

use crate::parser::diagnostics::Diagnostic;
use crate::parser::language::LanguageId;
use crate::parser::node::{NodeKind, SyntaxNode};
use crate::parser::position::{Position, TextRange};
use crate::parser::token::Token;
use serde::{Deserialize, Serialize};

/// High-level symbol metadata for LSP symbol search and outline navigation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SymbolInformation {
    pub name: String,
    pub kind: String,
    pub range: TextRange,
    pub container_name: Option<String>,
}

/// Represents the complete syntax tree output of a parse operation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyntaxTree {
    /// Language format of the parsed tree.
    pub language: LanguageId,
    /// Root AST syntax node.
    pub root: SyntaxNode,
    /// Extracted token stream.
    pub tokens: Vec<Token>,
    /// Syntax diagnostics / errors detected during parsing.
    pub diagnostics: Vec<Diagnostic>,
    /// FNV/SipHash of the source text used for cache validation.
    pub source_hash: u64,
    /// Length of source text in bytes.
    pub source_length: usize,
}

impl SyntaxTree {
    /// Creates a new `SyntaxTree`.
    pub fn new(
        language: LanguageId,
        root: SyntaxNode,
        tokens: Vec<Token>,
        diagnostics: Vec<Diagnostic>,
        source_hash: u64,
        source_length: usize,
    ) -> Self {
        Self {
            language,
            root,
            tokens,
            diagnostics,
            source_hash,
            source_length,
        }
    }

    /// Returns reference to the root node.
    pub fn root_node(&self) -> &SyntaxNode {
        &self.root
    }

    /// Finds the syntax node at the given position.
    pub fn find_node_at(&self, pos: Position) -> Option<&SyntaxNode> {
        self.root.find_node_at(pos)
    }

    /// Checks if the parsed tree contains any diagnostic syntax errors.
    pub fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|d| d.severity == crate::parser::diagnostics::DiagnosticSeverity::Error)
            || self.root.is_error
    }

    /// Extracts structural symbols for document outline / LSP support.
    pub fn symbols(&self) -> Vec<SymbolInformation> {
        let mut symbols = Vec::new();
        self.root.walk(|node| match &node.kind {
            NodeKind::Section | NodeKind::Pair | NodeKind::Block | NodeKind::Statement
                if !node.name.is_empty() && node.name != "document" => {
                    symbols.push(SymbolInformation {
                        name: node.name.clone(),
                        kind: format!("{:?}", node.kind),
                        range: node.range,
                        container_name: None,
                    });
                }
            _ => {}
        });
        symbols
    }
}
