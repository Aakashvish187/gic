//! Bash and Linux shell script parser implementation.

use crate::parser::diagnostics::Diagnostic;
use crate::parser::errors::ParseError;
use crate::parser::language::LanguageId;
use crate::parser::node::{NodeKind, SyntaxNode};
use crate::parser::parser_trait::LanguageParser;
use crate::parser::position::{Position, TextRange};
use crate::parser::token::{Token, TokenKind};
use crate::parser::tree::SyntaxTree;
use crate::parser::utils::{byte_offset_to_position, compute_line_offsets, hash_source};

const BASH_KEYWORDS: &[&str] = &[
    "if", "then", "else", "elif", "fi", "for", "in", "do", "done", "while", "until", "case",
    "esac", "function", "select", "time", "return", "exit",
];

/// Production Bash / Shell script parser.
#[derive(Debug, Default, Clone)]
pub struct BashParser;

impl BashParser {
    pub fn new() -> Self {
        Self
    }
}

impl LanguageParser for BashParser {
    fn language(&self) -> LanguageId {
        LanguageId::Bash
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
            } else if trimmed.starts_with('#') {
                tokens.push(Token::new(TokenKind::Comment, range, line));
                child_nodes.push(SyntaxNode::new(
                    NodeKind::Comment,
                    "bash_comment",
                    range,
                    vec![],
                ));
            } else {
                let parts: Vec<&str> = trimmed.split_whitespace().collect();
                let first_word = parts.first().copied().unwrap_or("");

                if BASH_KEYWORDS.contains(&first_word) {
                    tokens.push(Token::new(TokenKind::Keyword, range, first_word));
                    child_nodes.push(SyntaxNode::new(
                        NodeKind::Statement,
                        format!("keyword_{}", first_word),
                        range,
                        vec![],
                    ));
                } else if let Some((var, val)) = trimmed.split_once('=') {
                    tokens.push(Token::new(TokenKind::Variable, range, var.trim()));
                    tokens.push(Token::new(TokenKind::Operator, range, "="));
                    tokens.push(Token::new(TokenKind::String, range, val.trim()));

                    child_nodes.push(SyntaxNode::new(
                        NodeKind::Pair,
                        format!("variable_assignment: {}", var.trim()),
                        range,
                        vec![],
                    ));
                } else {
                    tokens.push(Token::new(TokenKind::Function, range, first_word));
                    for arg in parts.iter().skip(1) {
                        if arg.starts_with('$') {
                            tokens.push(Token::new(TokenKind::Variable, range, *arg));
                        } else if arg.starts_with('-') {
                            tokens.push(Token::new(TokenKind::Identifier, range, *arg));
                        } else {
                            tokens.push(Token::new(TokenKind::String, range, *arg));
                        }
                    }

                    child_nodes.push(SyntaxNode::new(
                        NodeKind::Statement,
                        format!("command: {}", first_word),
                        range,
                        vec![],
                    ));
                }
            }

            // Check for unclosed quote errors
            let double_quotes = line.chars().filter(|c| *c == '"').count();
            let single_quotes = line.chars().filter(|c| *c == '\'').count();
            if double_quotes % 2 != 0 || single_quotes % 2 != 0 {
                diagnostics.push(Diagnostic::warning(
                    range,
                    format!("Unclosed quote detected on line {}", line_idx + 1),
                    "bash-parser",
                ));
            }

            current_offset += line_len + 1;
        }

        let doc_end_pos = byte_offset_to_position(&line_offsets, source.len());
        let doc_range = TextRange::new(Position::zero(), doc_end_pos);
        let root = SyntaxNode::new(NodeKind::Document, "bash_document", doc_range, child_nodes);

        Ok(SyntaxTree::new(
            LanguageId::Bash,
            root,
            tokens,
            diagnostics,
            hash_source(source),
            source.len(),
        ))
    }
}
