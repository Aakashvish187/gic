//! Dockerfile parser implementation.

use crate::parser::diagnostics::Diagnostic;
use crate::parser::errors::ParseError;
use crate::parser::language::LanguageId;
use crate::parser::node::{NodeKind, SyntaxNode};
use crate::parser::parser_trait::LanguageParser;
use crate::parser::position::{Position, TextRange};
use crate::parser::token::{Token, TokenKind};
use crate::parser::tree::SyntaxTree;
use crate::parser::utils::{byte_offset_to_position, compute_line_offsets, hash_source};

const DOCKERFILE_INSTRUCTIONS: &[&str] = &[
    "FROM",
    "RUN",
    "CMD",
    "LABEL",
    "MAINTAINER",
    "EXPOSE",
    "ENV",
    "ADD",
    "COPY",
    "ENTRYPOINT",
    "VOLUME",
    "USER",
    "WORKDIR",
    "ARG",
    "ONBUILD",
    "STOPSIGNAL",
    "HEALTHCHECK",
    "SHELL",
];

/// Production Dockerfile parser.
#[derive(Debug, Default, Clone)]
pub struct DockerfileParser;

impl DockerfileParser {
    pub fn new() -> Self {
        Self
    }
}

impl LanguageParser for DockerfileParser {
    fn language(&self) -> LanguageId {
        LanguageId::Dockerfile
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
                let start_pos = byte_offset_to_position(&line_offsets, line_start_offset);
                let end_pos = byte_offset_to_position(&line_offsets, line_start_offset + line_len);
                tokens.push(Token::new(
                    TokenKind::Whitespace,
                    TextRange::new(start_pos, end_pos),
                    line,
                ));
            } else if trimmed.starts_with('#') {
                let start_pos = byte_offset_to_position(&line_offsets, line_start_offset);
                let end_pos = byte_offset_to_position(&line_offsets, line_start_offset + line_len);
                let range = TextRange::new(start_pos, end_pos);

                tokens.push(Token::new(TokenKind::Comment, range, line));
                child_nodes.push(SyntaxNode::new(NodeKind::Comment, "comment", range, vec![]));
            } else {
                let parts: Vec<&str> = trimmed.split_whitespace().collect();
                let first_word = parts.first().copied().unwrap_or("").to_uppercase();

                let start_pos = byte_offset_to_position(&line_offsets, line_start_offset);
                let end_pos = byte_offset_to_position(&line_offsets, line_start_offset + line_len);
                let range = TextRange::new(start_pos, end_pos);

                if DOCKERFILE_INSTRUCTIONS.contains(&first_word.as_str()) {
                    tokens.push(Token::new(TokenKind::Keyword, range, &first_word));
                    let args = trimmed[parts[0].len()..].trim();
                    if !args.is_empty() {
                        tokens.push(Token::new(TokenKind::String, range, args));
                    }

                    let instruction_node = SyntaxNode::new(
                        NodeKind::Statement,
                        format!("instruction_{}", first_word.to_lowercase()),
                        range,
                        vec![
                            SyntaxNode::new(NodeKind::Key, "instruction", range, vec![]),
                            SyntaxNode::new(NodeKind::Value, "arguments", range, vec![]),
                        ],
                    );
                    child_nodes.push(instruction_node);
                } else {
                    diagnostics.push(Diagnostic::error(
                        range,
                        format!(
                            "Unknown Dockerfile instruction on line {}: '{}'",
                            line_idx + 1,
                            first_word
                        ),
                        "dockerfile-parser",
                    ));
                    child_nodes.push(SyntaxNode::error(
                        format!("unknown_instruction: {}", first_word),
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
            "dockerfile_document",
            doc_range,
            child_nodes,
        );

        Ok(SyntaxTree::new(
            LanguageId::Dockerfile,
            root,
            tokens,
            diagnostics,
            hash_source(source),
            source.len(),
        ))
    }
}
