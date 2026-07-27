//! Duplicate Key Detection Engine for YAML.
//!
//! Traverses YAML mappings recursively to detect duplicate keys defined within the same
//! mapping context scope.

use std::collections::HashMap;

use crate::yaml::parser::{Position, Span, YamlAST, YamlMapping, YamlNode, YamlValue};

/// Description of a duplicate key collision in a YAML mapping.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DuplicateKeyIssue {
    /// Key identifier string.
    pub key: String,
    /// Line of the duplicate key definition.
    pub duplicate_line: usize,
    /// Column of the duplicate key definition.
    pub duplicate_column: usize,
    /// Line of the original key definition in the same mapping scope.
    pub original_line: usize,
    /// Column of the original key definition.
    pub original_column: usize,
    /// Span of the duplicate key node.
    pub duplicate_span: Span,
    /// Span of the first occurrence of the key.
    pub original_span: Span,
}

/// Detector for identifying duplicate keys in YAML mappings.
#[derive(Debug, Clone, Default)]
pub struct DuplicateKeyDetector;

impl DuplicateKeyDetector {
    /// Creates a new DuplicateKeyDetector.
    pub fn new() -> Self {
        Self
    }

    /// Analyzes a full YAML AST and collects all duplicate key issues.
    pub fn detect(&self, ast: &YamlAST) -> Vec<DuplicateKeyIssue> {
        let mut issues = Vec::new();
        for doc in &ast.documents {
            if let Some(ref root) = doc.root {
                self.inspect_node(root, &mut issues);
            }
        }
        issues
    }

    fn inspect_node(&self, node: &YamlNode, issues: &mut Vec<DuplicateKeyIssue>) {
        match &node.value {
            YamlValue::Mapping(mapping) => {
                self.inspect_mapping(mapping, issues);
            }
            YamlValue::Sequence(seq) => {
                for item in &seq.items {
                    self.inspect_node(item, issues);
                }
            }
            _ => {}
        }
    }

    fn inspect_mapping(&self, mapping: &YamlMapping, issues: &mut Vec<DuplicateKeyIssue>) {
        let mut seen_keys: HashMap<String, (Span, Position)> = HashMap::new();

        for pair in &mapping.pairs {
            let key_str = &pair.key.value;
            let key_span = pair.key.span;

            if let Some((orig_span, orig_pos)) = seen_keys.get(key_str) {
                issues.push(DuplicateKeyIssue {
                    key: key_str.clone(),
                    duplicate_line: key_span.start.line,
                    duplicate_column: key_span.start.column,
                    original_line: orig_pos.line,
                    original_column: orig_pos.column,
                    duplicate_span: key_span,
                    original_span: *orig_span,
                });
            } else {
                seen_keys.insert(key_str.clone(), (key_span, key_span.start));
            }

            // Recurse into pair value
            self.inspect_node(&pair.value, issues);
        }
    }
}
