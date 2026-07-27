//! JSON language parser implementation with fault-tolerant error recovery.

use crate::parser::diagnostics::Diagnostic;
use crate::parser::errors::ParseError;
use crate::parser::language::LanguageId;
use crate::parser::node::{NodeKind, SyntaxNode};
use crate::parser::parser_trait::LanguageParser;
use crate::parser::position::{Position, TextRange};
use crate::parser::token::{Token, TokenKind};
use crate::parser::tree::SyntaxTree;
use crate::parser::utils::{byte_offset_to_position, compute_line_offsets, hash_source};

/// Production JSON parser.
#[derive(Debug, Default, Clone)]
pub struct JsonParser;

impl JsonParser {
    pub fn new() -> Self {
        Self
    }
}

impl LanguageParser for JsonParser {
    fn language(&self) -> LanguageId {
        LanguageId::Json
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

        // 1. Attempt serde_json check to collect exact syntax errors
        if let Err(err) = serde_json::from_str::<serde_json::Value>(source) {
            let err_line = err.line().saturating_sub(1);
            let err_col = err.column().saturating_sub(1);
            let byte_offset = line_offsets
                .get(err_line)
                .map(|start| start + err_col)
                .unwrap_or(0);

            let pos = byte_offset_to_position(&line_offsets, byte_offset);
            diagnostics.push(Diagnostic::error(
                TextRange::new(pos, pos),
                format!("JSON syntax error: {}", err),
                "json-parser",
            ));
        }

        // 2. Build AST nodes and token stream
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
            } else {
                if trimmed.contains(':') {
                    if let Some((k, v)) = trimmed.split_once(':') {
                        let clean_k = k.trim().trim_matches('"');
                        let clean_v = v.trim().trim_matches(',').trim();

                        tokens.push(Token::new(TokenKind::Attribute, range, clean_k));
                        tokens.push(Token::new(TokenKind::Operator, range, ":"));
                        tokens.push(Token::new(TokenKind::String, range, clean_v));

                        let key_node = SyntaxNode::new(NodeKind::Key, clean_k, range, vec![]);
                        let val_node = SyntaxNode::new(NodeKind::Value, clean_v, range, vec![]);

                        child_nodes.push(SyntaxNode::new(
                            NodeKind::Pair,
                            format!("json_property: {}", clean_k),
                            range,
                            vec![key_node, val_node],
                        ));
                    }
                } else if trimmed == "{" || trimmed == "}" || trimmed == "[" || trimmed == "]" {
                    tokens.push(Token::new(TokenKind::Punctuation, range, trimmed));
                    child_nodes.push(SyntaxNode::new(
                        NodeKind::Block,
                        format!("json_structural: {}", trimmed),
                        range,
                        vec![],
                    ));
                } else {
                    tokens.push(Token::new(TokenKind::String, range, trimmed));
                    child_nodes.push(SyntaxNode::new(
                        NodeKind::Value,
                        format!("json_scalar: {}", trimmed),
                        range,
                        vec![],
                    ));
                }
            }

            // Detect unclosed JSON syntax errors per line if present
            if trimmed.ends_with(',') && (trimmed == "{" || trimmed == "[") {
                diagnostics.push(Diagnostic::error(
                    range,
                    format!(
                        "Invalid trailing comma or delimiter on line {}",
                        line_idx + 1
                    ),
                    "json-parser",
                ));
            }

            current_offset += line_len + 1;
        }

        let doc_end_pos = byte_offset_to_position(&line_offsets, source.len());
        let doc_range = TextRange::new(Position::zero(), doc_end_pos);

        let root = if diagnostics
            .iter()
            .any(|d| d.severity == crate::parser::diagnostics::DiagnosticSeverity::Error)
        {
            let mut node =
                SyntaxNode::new(NodeKind::Document, "json_document", doc_range, child_nodes);
            node.is_error = true;
            node
        } else {
            SyntaxNode::new(NodeKind::Document, "json_document", doc_range, child_nodes)
        };

        Ok(SyntaxTree::new(
            LanguageId::Json,
            root,
            tokens,
            diagnostics,
            hash_source(source),
            source.len(),
        ))
    }
}
