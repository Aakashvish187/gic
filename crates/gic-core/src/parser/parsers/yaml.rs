//! YAML language parser implementation with resilient error recovery.

use crate::parser::diagnostics::Diagnostic;
use crate::parser::errors::ParseError;
use crate::parser::language::LanguageId;
use crate::parser::node::{NodeKind, SyntaxNode};
use crate::parser::parser_trait::LanguageParser;
use crate::parser::position::{Position, TextRange};
use crate::parser::token::{Token, TokenKind};
use crate::parser::tree::SyntaxTree;
use crate::parser::utils::{byte_offset_to_position, compute_line_offsets, hash_source};

/// Production-grade YAML parser.
#[derive(Debug, Default, Clone)]
pub struct YamlParser;

impl YamlParser {
    pub fn new() -> Self {
        Self
    }
}

impl LanguageParser for YamlParser {
    fn language(&self) -> LanguageId {
        LanguageId::Yaml
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

        let mut current_offset = 0;

        for (line_idx, line) in source.lines().enumerate() {
            let line_start_offset = current_offset;
            let line_len = line.len();

            let trimmed = line.trim();

            if trimmed.is_empty() {
                // Whitespace line
                let start_pos = byte_offset_to_position(&line_offsets, line_start_offset);
                let end_pos = byte_offset_to_position(&line_offsets, line_start_offset + line_len);
                tokens.push(Token::new(
                    TokenKind::Whitespace,
                    TextRange::new(start_pos, end_pos),
                    line,
                ));
            } else if trimmed.starts_with('#') {
                // Comment line
                let start_pos = byte_offset_to_position(&line_offsets, line_start_offset);
                let end_pos = byte_offset_to_position(&line_offsets, line_start_offset + line_len);
                let range = TextRange::new(start_pos, end_pos);

                tokens.push(Token::new(TokenKind::Comment, range, line));
                child_nodes.push(SyntaxNode::new(NodeKind::Comment, "comment", range, vec![]));
            } else if trimmed.starts_with("- ") || trimmed == "-" {
                // Sequence item
                let start_pos = byte_offset_to_position(&line_offsets, line_start_offset);
                let end_pos = byte_offset_to_position(&line_offsets, line_start_offset + line_len);
                let range = TextRange::new(start_pos, end_pos);

                tokens.push(Token::new(TokenKind::Punctuation, range, "-"));
                tokens.push(Token::new(
                    TokenKind::String,
                    range,
                    trimmed.trim_start_matches('-').trim(),
                ));

                child_nodes.push(SyntaxNode::new(
                    NodeKind::ListItem,
                    "sequence_item",
                    range,
                    vec![],
                ));
            } else if let Some((key, val)) = trimmed.split_once(':') {
                // Mapping key-value pair
                let start_pos = byte_offset_to_position(&line_offsets, line_start_offset);
                let end_pos = byte_offset_to_position(&line_offsets, line_start_offset + line_len);
                let range = TextRange::new(start_pos, end_pos);

                let key_node = SyntaxNode::new(
                    NodeKind::Key,
                    "yaml_key",
                    range,
                    vec![SyntaxNode::new(
                        NodeKind::Custom("identifier".into()),
                        key.trim(),
                        range,
                        vec![],
                    )],
                );

                let val_node = SyntaxNode::new(
                    NodeKind::Value,
                    "yaml_value",
                    range,
                    vec![SyntaxNode::new(
                        NodeKind::Custom("scalar".into()),
                        val.trim(),
                        range,
                        vec![],
                    )],
                );

                tokens.push(Token::new(TokenKind::Attribute, range, key.trim()));
                tokens.push(Token::new(TokenKind::Operator, range, ":"));
                if !val.trim().is_empty() {
                    tokens.push(Token::new(TokenKind::String, range, val.trim()));
                }

                child_nodes.push(SyntaxNode::new(
                    NodeKind::Pair,
                    format!("key_value: {}", key.trim()),
                    range,
                    vec![key_node, val_node],
                ));
            } else {
                // Unrecognized or syntax error line
                let start_pos = byte_offset_to_position(&line_offsets, line_start_offset);
                let end_pos = byte_offset_to_position(&line_offsets, line_start_offset + line_len);
                let range = TextRange::new(start_pos, end_pos);

                diagnostics.push(Diagnostic::error(
                    range,
                    format!("Malformed YAML line {}: '{}'", line_idx + 1, trimmed),
                    "yaml-parser",
                ));

                child_nodes.push(SyntaxNode::error(
                    format!("invalid_yaml_line: {}", trimmed),
                    range,
                ));
            }

            // Move offset past content + newline
            current_offset += line_len + 1;
        }

        let doc_end_pos = byte_offset_to_position(&line_offsets, source.len());
        let doc_range = TextRange::new(Position::zero(), doc_end_pos);
        let root = SyntaxNode::new(NodeKind::Document, "yaml_document", doc_range, child_nodes);

        Ok(SyntaxTree::new(
            LanguageId::Yaml,
            root,
            tokens,
            diagnostics,
            hash_source(source),
            source.len(),
        ))
    }
}
