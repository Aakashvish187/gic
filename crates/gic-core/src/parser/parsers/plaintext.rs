//! Plain text fallback language parser implementation.

use crate::parser::errors::ParseError;
use crate::parser::language::LanguageId;
use crate::parser::node::{NodeKind, SyntaxNode};
use crate::parser::parser_trait::LanguageParser;
use crate::parser::position::{Position, TextRange};
use crate::parser::token::{Token, TokenKind};
use crate::parser::tree::SyntaxTree;
use crate::parser::utils::{byte_offset_to_position, compute_line_offsets, hash_source};

/// Production Plain Text fallback parser.
#[derive(Debug, Default, Clone)]
pub struct PlainTextParser;

impl PlainTextParser {
    pub fn new() -> Self {
        Self
    }
}

impl LanguageParser for PlainTextParser {
    fn language(&self) -> LanguageId {
        LanguageId::PlainText
    }

    fn parse(
        &self,
        source: &str,
        _old_tree: Option<&SyntaxTree>,
    ) -> Result<SyntaxTree, ParseError> {
        let line_offsets = compute_line_offsets(source);
        let mut tokens = Vec::new();
        let mut child_nodes = Vec::new();

        let mut current_offset = 0;

        for line in source.lines() {
            let line_start_offset = current_offset;
            let line_len = line.len();

            let start_pos = byte_offset_to_position(&line_offsets, line_start_offset);
            let end_pos = byte_offset_to_position(&line_offsets, line_start_offset + line_len);
            let range = TextRange::new(start_pos, end_pos);

            if line.trim().is_empty() {
                tokens.push(Token::new(TokenKind::Whitespace, range, line));
            } else {
                tokens.push(Token::new(TokenKind::String, range, line));
                child_nodes.push(SyntaxNode::new(
                    NodeKind::Statement,
                    "text_line",
                    range,
                    vec![],
                ));
            }

            current_offset += line_len + 1;
        }

        let doc_end_pos = byte_offset_to_position(&line_offsets, source.len());
        let doc_range = TextRange::new(Position::zero(), doc_end_pos);
        let root = SyntaxNode::new(
            NodeKind::Document,
            "plaintext_document",
            doc_range,
            child_nodes,
        );

        Ok(SyntaxTree::new(
            LanguageId::PlainText,
            root,
            tokens,
            Vec::new(), // Plain text produces no syntax diagnostics
            hash_source(source),
            source.len(),
        ))
    }
}
