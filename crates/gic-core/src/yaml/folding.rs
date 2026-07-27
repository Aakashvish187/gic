//! YAML Code Folding Engine.
//!
//! Calculates collapsible line ranges for mappings, sequences, block scalars (`|`, `>`),
//! multi-line comments, and multi-document boundaries.

use crate::yaml::parser::{Span, YamlAST, YamlNode, YamlValue};

/// Category of collapsible code block.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FoldingKind {
    /// Mapping block structure.
    Mapping,
    /// Sequence array block structure.
    Sequence,
    /// Multiline block scalar (`|` or `>`).
    BlockScalar,
    /// Consecutive line comment block.
    Comment,
    /// Document boundary stream (`---`).
    Document,
}

/// Collapsible code range descriptor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FoldingRange {
    /// Type of collapsible section.
    pub kind: FoldingKind,
    /// 1-indexed start line of fold.
    pub start_line: usize,
    /// 1-indexed end line of fold.
    pub end_line: usize,
    /// Span location of the folding region.
    pub span: Span,
}

/// Code folding range calculator for YAML AST.
#[derive(Debug, Clone, Default)]
pub struct YamlFoldingEngine;

impl YamlFoldingEngine {
    /// Creates a new YamlFoldingEngine.
    pub fn new() -> Self {
        Self
    }

    /// Computes all collapsible folding ranges for a given `YamlAST`.
    pub fn compute_folding_ranges(&self, ast: &YamlAST) -> Vec<FoldingRange> {
        let mut ranges = Vec::new();

        for doc in &ast.documents {
            // Fold multi-line documents
            if doc.span.end.line > doc.span.start.line {
                ranges.push(FoldingRange {
                    kind: FoldingKind::Document,
                    start_line: doc.span.start.line,
                    end_line: doc.span.end.line,
                    span: doc.span,
                });
            }

            if let Some(ref root) = doc.root {
                self.inspect_node(root, &mut ranges);
            }
        }

        // Fold consecutive comments
        self.compute_comment_folds(&ast.comments, &mut ranges);

        ranges
    }

    fn inspect_node(&self, node: &YamlNode, ranges: &mut Vec<FoldingRange>) {
        let start_line = node.span.start.line;
        let end_line = node.span.end.line;

        if end_line > start_line {
            let kind = match &node.value {
                YamlValue::Mapping(_) => Some(FoldingKind::Mapping),
                YamlValue::Sequence(_) => Some(FoldingKind::Sequence),
                YamlValue::Scalar(s)
                    if matches!(
                        s.style,
                        crate::yaml::parser::YamlScalarStyle::LiteralBlock
                            | crate::yaml::parser::YamlScalarStyle::FoldedBlock
                    ) =>
                {
                    Some(FoldingKind::BlockScalar)
                }
                _ => None,
            };

            if let Some(k) = kind {
                ranges.push(FoldingRange {
                    kind: k,
                    start_line,
                    end_line,
                    span: node.span,
                });
            }
        }

        match &node.value {
            YamlValue::Mapping(mapping) => {
                for pair in &mapping.pairs {
                    self.inspect_node(&pair.value, ranges);
                }
            }
            YamlValue::Sequence(seq) => {
                for item in &seq.items {
                    self.inspect_node(item, ranges);
                }
            }
            _ => {}
        }
    }

    fn compute_comment_folds(
        &self,
        comments: &[crate::yaml::parser::YamlComment],
        ranges: &mut Vec<FoldingRange>,
    ) {
        if comments.len() < 2 {
            return;
        }

        let mut start_idx = 0;
        for i in 1..comments.len() {
            let prev = &comments[i - 1];
            let curr = &comments[i];

            if curr.span.start.line != prev.span.start.line + 1 || curr.is_inline || prev.is_inline
            {
                if i - 1 > start_idx {
                    let first = &comments[start_idx];
                    let last = &comments[i - 1];
                    ranges.push(FoldingRange {
                        kind: FoldingKind::Comment,
                        start_line: first.span.start.line,
                        end_line: last.span.end.line,
                        span: Span::new(first.span.start, last.span.end),
                    });
                }
                start_idx = i;
            }
        }

        if comments.len() - 1 > start_idx {
            let first = &comments[start_idx];
            let last = comments.last().unwrap();
            ranges.push(FoldingRange {
                kind: FoldingKind::Comment,
                start_line: first.span.start.line,
                end_line: last.span.end.line,
                span: Span::new(first.span.start, last.span.end),
            });
        }
    }
}
