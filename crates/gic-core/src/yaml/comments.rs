//! YAML Comment Preservation and Association Engine.
//!
//! Extracts, categorizes, and attaches inline, header, block, and trailing comments (`# comment`)
//! to closest AST nodes for comment-preserving formatting and doc extraction.

use crate::yaml::parser::{YamlAST, YamlComment, YamlNode, YamlValue};

/// Category of comment position relative to code structure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CommentPlacement {
    /// Header comment at the start of document before code.
    Header,
    /// Line comment appearing above a code block.
    Leading,
    /// Inline comment on the same line after code.
    Inline,
    /// Trailing comment at the end of document.
    Trailing,
}

/// Comment association binder.
#[derive(Debug, Clone, Default)]
pub struct CommentHandler;

impl CommentHandler {
    /// Creates a new CommentHandler.
    pub fn new() -> Self {
        Self
    }

    /// Attaches isolated comments from `ast` to the closest corresponding `YamlNode`s.
    pub fn bind_comments(&self, ast: &mut YamlAST) {
        let comments = ast.comments.clone();
        for doc in &mut ast.documents {
            if let Some(ref mut root) = doc.root {
                self.bind_to_node(root, &comments);
            }
        }
    }

    fn bind_to_node(&self, node: &mut YamlNode, comments: &[YamlComment]) {
        for comment in comments {
            // Check leading comment
            if !comment.is_inline && comment.span.end.line + 1 == node.span.start.line
                && !node.leading_comments.contains(comment) {
                    node.leading_comments.push(comment.clone());
                }
            // Check trailing/inline comment on same line
            if comment.is_inline && comment.span.start.line == node.span.start.line
                && node.trailing_comment.is_none() {
                    node.trailing_comment = Some(comment.clone());
                }
        }

        match &mut node.value {
            YamlValue::Mapping(mapping) => {
                for pair in &mut mapping.pairs {
                    self.bind_to_node(&mut pair.value, comments);
                }
            }
            YamlValue::Sequence(seq) => {
                for item in &mut seq.items {
                    self.bind_to_node(item, comments);
                }
            }
            _ => {}
        }
    }
}
