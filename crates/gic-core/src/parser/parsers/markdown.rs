//! Markdown language parser implementation.

use crate::parser::diagnostics::Diagnostic;
use crate::parser::errors::ParseError;
use crate::parser::language::LanguageId;
use crate::parser::node::{NodeKind, SyntaxNode};
use crate::parser::parser_trait::LanguageParser;
use crate::parser::position::{Position, TextRange};
use crate::parser::token::{Token, TokenKind};
use crate::parser::tree::SyntaxTree;
use crate::parser::utils::{byte_offset_to_position, compute_line_offsets, hash_source};

/// Production Markdown parser.
#[derive(Debug, Default, Clone)]
pub struct MarkdownParser;

impl MarkdownParser {
    pub fn new() -> Self {
        Self
    }
}

impl LanguageParser for MarkdownParser {
    fn language(&self) -> LanguageId {
        LanguageId::Markdown
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

        let mut in_code_block = false;
        let mut current_offset = 0;

        for line in source.lines() {
            let line_start_offset = current_offset;
            let line_len = line.len();
            let trimmed = line.trim();

            let start_pos = byte_offset_to_position(&line_offsets, line_start_offset);
            let end_pos = byte_offset_to_position(&line_offsets, line_start_offset + line_len);
            let range = TextRange::new(start_pos, end_pos);

            if trimmed.starts_with("```") {
                in_code_block = !in_code_block;
                tokens.push(Token::new(TokenKind::Keyword, range, trimmed));
                child_nodes.push(SyntaxNode::new(
                    NodeKind::Block,
                    "code_fence",
                    range,
                    vec![],
                ));
            } else if in_code_block {
                tokens.push(Token::new(TokenKind::String, range, line));
                child_nodes.push(SyntaxNode::new(
                    NodeKind::Statement,
                    "code_line",
                    range,
                    vec![],
                ));
            } else if trimmed.starts_with('#') {
                let level = trimmed.chars().take_while(|c| *c == '#').count();
                let title = trimmed[level..].trim();

                tokens.push(Token::new(TokenKind::Keyword, range, &trimmed[..level]));
                tokens.push(Token::new(TokenKind::Identifier, range, title));

                child_nodes.push(SyntaxNode::new(
                    NodeKind::Section,
                    format!("heading_h{}: {}", level, title),
                    range,
                    vec![],
                ));
            } else if trimmed.starts_with("- ")
                || trimmed.starts_with("* ")
                || trimmed.starts_with("+ ")
            {
                tokens.push(Token::new(TokenKind::Punctuation, range, &trimmed[..1]));
                tokens.push(Token::new(TokenKind::String, range, &trimmed[2..]));

                child_nodes.push(SyntaxNode::new(
                    NodeKind::ListItem,
                    "list_item",
                    range,
                    vec![],
                ));
            } else if let Some(rest) = trimmed.strip_prefix('>') {
                tokens.push(Token::new(TokenKind::Punctuation, range, ">"));
                tokens.push(Token::new(TokenKind::String, range, rest.trim()));

                child_nodes.push(SyntaxNode::new(
                    NodeKind::Block,
                    "blockquote",
                    range,
                    vec![],
                ));
            } else if trimmed.is_empty() {
                tokens.push(Token::new(TokenKind::Whitespace, range, line));
            } else {
                tokens.push(Token::new(TokenKind::String, range, line));
                child_nodes.push(SyntaxNode::new(
                    NodeKind::Statement,
                    "paragraph_text",
                    range,
                    vec![],
                ));
            }

            current_offset += line_len + 1;
        }

        if in_code_block {
            let doc_end_pos = byte_offset_to_position(&line_offsets, source.len());
            diagnostics.push(Diagnostic::warning(
                TextRange::new(doc_end_pos, doc_end_pos),
                "Unclosed Markdown code fence block at EOF",
                "markdown-parser",
            ));
        }

        let doc_end_pos = byte_offset_to_position(&line_offsets, source.len());
        let doc_range = TextRange::new(Position::zero(), doc_end_pos);
        let root = SyntaxNode::new(
            NodeKind::Document,
            "markdown_document",
            doc_range,
            child_nodes,
        );

        Ok(SyntaxTree::new(
            LanguageId::Markdown,
            root,
            tokens,
            diagnostics,
            hash_source(source),
            source.len(),
        ))
    }
}
