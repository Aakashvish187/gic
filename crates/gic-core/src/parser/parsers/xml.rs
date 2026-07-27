//! XML language parser implementation.

use crate::parser::diagnostics::Diagnostic;
use crate::parser::errors::ParseError;
use crate::parser::language::LanguageId;
use crate::parser::node::{NodeKind, SyntaxNode};
use crate::parser::parser_trait::LanguageParser;
use crate::parser::position::{Position, TextRange};
use crate::parser::token::{Token, TokenKind};
use crate::parser::tree::SyntaxTree;
use crate::parser::utils::{byte_offset_to_position, compute_line_offsets, hash_source};

/// Production XML parser.
#[derive(Debug, Default, Clone)]
pub struct XmlParser;

impl XmlParser {
    pub fn new() -> Self {
        Self
    }
}

impl LanguageParser for XmlParser {
    fn language(&self) -> LanguageId {
        LanguageId::Xml
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
        let mut tag_stack: Vec<(String, usize)> = Vec::new();

        for (line_idx, line) in source.lines().enumerate() {
            let line_start_offset = current_offset;
            let line_len = line.len();
            let trimmed = line.trim();

            let start_pos = byte_offset_to_position(&line_offsets, line_start_offset);
            let end_pos = byte_offset_to_position(&line_offsets, line_start_offset + line_len);
            let range = TextRange::new(start_pos, end_pos);

            if trimmed.is_empty() {
                tokens.push(Token::new(TokenKind::Whitespace, range, line));
            } else if trimmed.starts_with("<!--") && trimmed.ends_with("-->") {
                tokens.push(Token::new(TokenKind::Comment, range, line));
                child_nodes.push(SyntaxNode::new(
                    NodeKind::Comment,
                    "xml_comment",
                    range,
                    vec![],
                ));
            } else if trimmed.starts_with("</") {
                if let Some(close_idx) = trimmed.find('>') {
                    let tag_name = trimmed[2..close_idx].trim().to_string();
                    tokens.push(Token::new(TokenKind::Tag, range, &trimmed[..=close_idx]));

                    if let Some((open_tag, _)) = tag_stack.pop() {
                        if open_tag != tag_name {
                            diagnostics.push(Diagnostic::error(
                                range,
                                format!("Mismatched XML closing tag '</{}>', expected '</{}>' on line {}", tag_name, open_tag, line_idx + 1),
                                "xml-parser",
                            ));
                        }
                    } else {
                        diagnostics.push(Diagnostic::error(
                            range,
                            format!(
                                "Unmatched XML closing tag '</{}>' on line {}",
                                tag_name,
                                line_idx + 1
                            ),
                            "xml-parser",
                        ));
                    }

                    child_nodes.push(SyntaxNode::new(
                        NodeKind::Block,
                        format!("close_tag_{}", tag_name),
                        range,
                        vec![],
                    ));
                } else {
                    diagnostics.push(Diagnostic::error(
                        range,
                        format!("Malformed closing tag on line {}", line_idx + 1),
                        "xml-parser",
                    ));
                }
            } else if trimmed.starts_with('<') {
                if let Some(close_idx) = trimmed.find('>') {
                    let tag_content = trimmed[1..close_idx].trim();
                    let is_self_closing = tag_content.ends_with('/');
                    let clean_content = if is_self_closing {
                        tag_content.trim_end_matches('/').trim()
                    } else {
                        tag_content
                    };

                    let tag_name = clean_content.split_whitespace().next().unwrap_or("");
                    tokens.push(Token::new(TokenKind::Tag, range, &trimmed[..=close_idx]));

                    if !is_self_closing && !tag_name.starts_with('?') && !tag_name.starts_with('!')
                    {
                        tag_stack.push((tag_name.to_string(), line_idx + 1));
                    }

                    child_nodes.push(SyntaxNode::new(
                        NodeKind::Block,
                        format!("open_tag_{}", tag_name),
                        range,
                        vec![],
                    ));
                } else {
                    diagnostics.push(Diagnostic::error(
                        range,
                        format!("Unclosed XML tag header '<' on line {}", line_idx + 1),
                        "xml-parser",
                    ));
                }
            } else {
                tokens.push(Token::new(TokenKind::String, range, line));
                child_nodes.push(SyntaxNode::new(
                    NodeKind::Value,
                    "xml_text_node",
                    range,
                    vec![],
                ));
            }

            current_offset += line_len + 1;
        }

        for (unclosed_tag, line_num) in tag_stack {
            let doc_end_pos = byte_offset_to_position(&line_offsets, source.len());
            diagnostics.push(Diagnostic::error(
                TextRange::new(doc_end_pos, doc_end_pos),
                format!(
                    "Unclosed XML tag '<{}>' opened on line {}",
                    unclosed_tag, line_num
                ),
                "xml-parser",
            ));
        }

        let doc_end_pos = byte_offset_to_position(&line_offsets, source.len());
        let doc_range = TextRange::new(Position::zero(), doc_end_pos);
        let root = SyntaxNode::new(NodeKind::Document, "xml_document", doc_range, child_nodes);

        Ok(SyntaxTree::new(
            LanguageId::Xml,
            root,
            tokens,
            diagnostics,
            hash_source(source),
            source.len(),
        ))
    }
}
