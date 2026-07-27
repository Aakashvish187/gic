//! YAML Alias Resolution Engine.
//!
//! Evaluates alias references (`*alias_name`), checks resolution against anchor definitions,
//! and detects circular reference chains.

use std::collections::HashSet;

use crate::yaml::anchors::AnchorRegistry;
use crate::yaml::parser::{Span, YamlAST, YamlNode, YamlValue};

/// Defect identified during alias resolution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AliasIssueKind {
    /// Alias references an anchor that was never defined.
    UnresolvedAlias,
    /// Alias introduces a circular reference dependency chain.
    CircularReference,
}

/// Diagnostic report item for an alias issue.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AliasIssue {
    /// Category of alias defect.
    pub kind: AliasIssueKind,
    /// Target alias name.
    pub alias_name: String,
    /// Detailed message.
    pub message: String,
    /// Span of the alias token.
    pub span: Span,
}

/// Alias resolution engine.
#[derive(Debug, Clone, Default)]
pub struct AliasResolver;

impl AliasResolver {
    /// Creates a new AliasResolver.
    pub fn new() -> Self {
        Self
    }

    /// Validates all alias usages in a `YamlAST` against a given `AnchorRegistry`.
    pub fn resolve(&self, ast: &YamlAST, anchors: &AnchorRegistry) -> Vec<AliasIssue> {
        let mut issues = Vec::new();
        let mut visited_aliases = HashSet::new();

        for doc in &ast.documents {
            if let Some(ref root) = doc.root {
                self.check_node(root, anchors, &mut visited_aliases, &mut issues);
            }
        }
        issues
    }

    fn check_node(
        &self,
        node: &YamlNode,
        anchors: &AnchorRegistry,
        visited: &mut HashSet<String>,
        issues: &mut Vec<AliasIssue>,
    ) {
        match &node.value {
            YamlValue::Alias(alias_ref) => {
                let name = &alias_ref.name;

                if !anchors.contains(name) {
                    issues.push(AliasIssue {
                        kind: AliasIssueKind::UnresolvedAlias,
                        alias_name: name.clone(),
                        message: format!("Alias '*{name}' references an undefined anchor"),
                        span: alias_ref.span,
                    });
                } else if visited.contains(name) {
                    issues.push(AliasIssue {
                        kind: AliasIssueKind::CircularReference,
                        alias_name: name.clone(),
                        message: format!("Circular alias reference detected for '*{name}'"),
                        span: alias_ref.span,
                    });
                } else {
                    visited.insert(name.clone());
                }
            }
            YamlValue::Mapping(mapping) => {
                for pair in &mapping.pairs {
                    self.check_node(&pair.value, anchors, visited, issues);
                }
            }
            YamlValue::Sequence(seq) => {
                for item in &seq.items {
                    self.check_node(item, anchors, visited, issues);
                }
            }
            _ => {}
        }
    }
}
