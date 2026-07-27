//! TOML language parser implementation.

use crate::parser::diagnostics::Diagnostic;
use crate::parser::errors::ParseError;
use crate::parser::language::LanguageId;
use crate::parser::node::{NodeKind, SyntaxNode};
use crate::parser::parser_trait::LanguageParser;
use crate::parser::position::{Position, TextRange};
use crate::parser::token::{Token, TokenKind};
use crate::parser::tree::SyntaxTree;
use crate::parser::utils::{byte_offset_to_position, compute_line_offsets, hash_source};

/// Production TOML parser.
#[derive(Debug, Default, Clone)]
pub struct TomlParser;

impl TomlParser {
    pub fn new() -> Self {
        Self
    }
}

impl LanguageParser for TomlParser {
    fn language(&self) -> LanguageId {
        LanguageId::Toml
    }

    fn parse(
        &self,
        source: &str,
        _old_tree: Option<&SyntaxTree>,
    ) -> Result<SyntaxTree, ParseError> {
        let line_offsets = compute_line_offsets(source);
        let mut tokens = Vec::new();
        let mut diagnostics = Vec::new();
        let mut child_nodes = Vec::new();

        // Check TOML validity via standard library / crate
        if let Err(err) = toml::from_str::<toml::Value>(source) {
            diagnostics.push(Diagnostic::error(
                TextRange::new(
                    byte_offset_to_position(&line_offsets, 0),
                    byte_offset_to_position(&line_offsets, source.len()),
                ),
                format!("TOML parse error: {}", err),
                "toml-parser",
            ));
        }

        let mut current_offset = 0;
        for (line_idx, line) in source.lines().enumerate() {
            let line_start_offset = current_offset;
            let line_len = line.len();
            let trimmed = line.trim();

            let start_pos = byte_offset_to_position(&line_offsets, line_start_offset);
            let end_pos = byte_offset_to_position(&line_offsets, line_start_offset + line_len);
            let range = TextRange::new(start_pos, end_pos);

            if trimmed.is_empty() {
                tokens.push(Token::new(TokenKind::Whitespace, range, line));
            } else if trimmed.starts_with('#') {
                tokens.push(Token::new(TokenKind::Comment, range, line));
                child_nodes.push(SyntaxNode::new(
                    NodeKind::Comment,
                    "toml_comment",
                    range,
                    vec![],
                ));
            } else if trimmed.starts_with("[[") && trimmed.ends_with("]]") {
                let table_name = &trimmed[2..trimmed.len() - 2];
                tokens.push(Token::new(TokenKind::Keyword, range, trimmed));
                child_nodes.push(SyntaxNode::new(
                    NodeKind::Section,
                    format!("array_table: {}", table_name),
                    range,
                    vec![],
                ));
            } else if trimmed.starts_with('[') && trimmed.ends_with(']') {
                let table_name = &trimmed[1..trimmed.len() - 1];
                tokens.push(Token::new(TokenKind::Keyword, range, trimmed));
                child_nodes.push(SyntaxNode::new(
                    NodeKind::Section,
                    format!("table: {}", table_name),
                    range,
                    vec![],
                ));
            } else if let Some((key, val)) = trimmed.split_once('=') {
                tokens.push(Token::new(TokenKind::Attribute, range, key.trim()));
                tokens.push(Token::new(TokenKind::Operator, range, "="));
                tokens.push(Token::new(TokenKind::String, range, val.trim()));

                let key_node = SyntaxNode::new(NodeKind::Key, key.trim(), range, vec![]);
                let val_node = SyntaxNode::new(NodeKind::Value, val.trim(), range, vec![]);

                child_nodes.push(SyntaxNode::new(
                    NodeKind::Pair,
                    format!("key_value: {}", key.trim()),
                    range,
                    vec![key_node, val_node],
                ));
            } else {
                diagnostics.push(Diagnostic::error(
                    range,
                    format!(
                        "Malformed TOML entry on line {}: '{}'",
                        line_idx + 1,
                        trimmed
                    ),
                    "toml-parser",
                ));
                child_nodes.push(SyntaxNode::error(
                    format!("invalid_toml_entry: {}", trimmed),
                    range,
                ));
            }

            current_offset += line_len + 1;
        }

        let doc_end_pos = byte_offset_to_position(&line_offsets, source.len());
        let doc_range = TextRange::new(Position::zero(), doc_end_pos);

        let root = SyntaxNode::new(NodeKind::Document, "toml_document", doc_range, child_nodes);

        Ok(SyntaxTree::new(
            LanguageId::Toml,
            root,
            tokens,
            diagnostics,
            hash_source(source),
            source.len(),
        ))
    }
}
