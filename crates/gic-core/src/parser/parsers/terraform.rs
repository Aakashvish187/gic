//! Terraform / HCL language parser implementation.

use crate::parser::diagnostics::Diagnostic;
use crate::parser::errors::ParseError;
use crate::parser::language::LanguageId;
use crate::parser::node::{NodeKind, SyntaxNode};
use crate::parser::parser_trait::LanguageParser;
use crate::parser::position::{Position, TextRange};
use crate::parser::token::{Token, TokenKind};
use crate::parser::tree::SyntaxTree;
use crate::parser::utils::{byte_offset_to_position, compute_line_offsets, hash_source};

const HCL_BLOCK_TYPES: &[&str] = &[
    "resource",
    "variable",
    "output",
    "provider",
    "module",
    "terraform",
    "data",
    "locals",
];

/// Production Terraform (HCL) parser.
#[derive(Debug, Default, Clone)]
pub struct TerraformParser;

impl TerraformParser {
    pub fn new() -> Self {
        Self
    }
}

impl LanguageParser for TerraformParser {
    fn language(&self) -> LanguageId {
        LanguageId::Terraform
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

            let start_pos = byte_offset_to_position(&line_offsets, line_start_offset);
            let end_pos = byte_offset_to_position(&line_offsets, line_start_offset + line_len);
            let range = TextRange::new(start_pos, end_pos);

            if trimmed.is_empty() {
                tokens.push(Token::new(TokenKind::Whitespace, range, line));
            } else if trimmed.starts_with('#') || trimmed.starts_with("//") {
                tokens.push(Token::new(TokenKind::Comment, range, line));
                child_nodes.push(SyntaxNode::new(
                    NodeKind::Comment,
                    "hcl_comment",
                    range,
                    vec![],
                ));
            } else if trimmed == "}" || trimmed == "{" {
                tokens.push(Token::new(TokenKind::Punctuation, range, trimmed));
            } else {
                let parts: Vec<&str> = trimmed.split_whitespace().collect();
                let first_token = parts.first().copied().unwrap_or("");

                if HCL_BLOCK_TYPES.contains(&first_token) {
                    tokens.push(Token::new(TokenKind::Keyword, range, first_token));

                    let labels: Vec<&str> = parts
                        .iter()
                        .skip(1)
                        .filter(|p| **p != "{")
                        .copied()
                        .collect();
                    for label in &labels {
                        tokens.push(Token::new(TokenKind::String, range, *label));
                    }

                    child_nodes.push(SyntaxNode::new(
                        NodeKind::Block,
                        format!("hcl_block_{}", first_token),
                        range,
                        vec![],
                    ));
                } else if let Some((key, val)) = trimmed.split_once('=') {
                    tokens.push(Token::new(TokenKind::Attribute, range, key.trim()));
                    tokens.push(Token::new(TokenKind::Operator, range, "="));
                    tokens.push(Token::new(TokenKind::String, range, val.trim()));

                    let key_node = SyntaxNode::new(NodeKind::Key, "attribute_key", range, vec![]);
                    let val_node =
                        SyntaxNode::new(NodeKind::Value, "attribute_value", range, vec![]);

                    child_nodes.push(SyntaxNode::new(
                        NodeKind::Pair,
                        format!("attribute: {}", key.trim()),
                        range,
                        vec![key_node, val_node],
                    ));
                } else {
                    diagnostics.push(Diagnostic::warning(
                        range,
                        format!(
                            "Unrecognized HCL syntax on line {}: '{}'",
                            line_idx + 1,
                            trimmed
                        ),
                        "terraform-parser",
                    ));
                    child_nodes.push(SyntaxNode::error(
                        format!("unrecognized_hcl: {}", trimmed),
                        range,
                    ));
                }
            }

            current_offset += line_len + 1;
        }

        let doc_end_pos = byte_offset_to_position(&line_offsets, source.len());
        let doc_range = TextRange::new(Position::zero(), doc_end_pos);
        let root = SyntaxNode::new(
            NodeKind::Document,
            "terraform_document",
            doc_range,
            child_nodes,
        );

        Ok(SyntaxTree::new(
            LanguageId::Terraform,
            root,
            tokens,
            diagnostics,
            hash_source(source),
            source.len(),
        ))
    }
}
