//! Abstract Syntax Tree (AST) node representation and traversal interfaces.

use crate::parser::position::{Position, TextRange};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Categorization of AST Node kinds.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum NodeKind {
    /// Root document node
    Document,
    /// High-level section or section header (Markdown, INI)
    Section,
    /// Key or identifier in a key-value structure
    Key,
    /// Value in a key-value structure
    Value,
    /// Key-Value pair / Attribute
    Pair,
    /// List or Array block
    List,
    /// Individual item in a list or sequence
    ListItem,
    /// Code block, HCL block, or scope
    Block,
    /// Statement, Instruction, or Command line
    Statement,
    /// Comment node
    Comment,
    /// Error / Faulty syntax node
    Error,
    /// Custom node type name
    Custom(String),
}

impl fmt::Display for NodeKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NodeKind::Custom(name) => write!(f, "{}", name),
            _ => write!(f, "{:?}", self),
        }
    }
}

/// An immutable, hierarchical Syntax Node in a parsed AST tree.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyntaxNode {
    /// Functional category of node.
    pub kind: NodeKind,
    /// Detailed node type label (e.g. "mapping_pair", "hcl_resource_block").
    pub name: String,
    /// Source text range spanned by this node.
    pub range: TextRange,
    /// Whether this node represents a named production rule in grammar.
    pub is_named: bool,
    /// Whether this node was produced due to a syntax parsing error.
    pub is_error: bool,
    /// Whether this node was missing in source and auto-inserted during error recovery.
    pub is_missing: bool,
    /// Ordered child syntax nodes.
    pub children: Vec<SyntaxNode>,
}

impl SyntaxNode {
    /// Creates a new `SyntaxNode`.
    pub fn new(
        kind: NodeKind,
        name: impl Into<String>,
        range: TextRange,
        children: Vec<SyntaxNode>,
    ) -> Self {
        Self {
            kind,
            name: name.into(),
            range,
            is_named: true,
            is_error: false,
            is_missing: false,
            children,
        }
    }

    /// Creates an error syntax node.
    pub fn error(name: impl Into<String>, range: TextRange) -> Self {
        Self {
            kind: NodeKind::Error,
            name: name.into(),
            range,
            is_named: true,
            is_error: true,
            is_missing: false,
            children: Vec::new(),
        }
    }

    /// Extracts the exact source text slice corresponding to this node's range.
    pub fn text<'a>(&self, source: &'a str) -> &'a str {
        let start = self.range.start.byte_offset;
        let end = self.range.end.byte_offset;
        if start <= end && end <= source.len() {
            &source[start..end]
        } else {
            ""
        }
    }

    /// Finds the deepest node containing the given position.
    pub fn find_node_at(&self, pos: Position) -> Option<&SyntaxNode> {
        if !self.range.contains_position(pos) {
            return None;
        }

        for child in &self.children {
            if let Some(found) = child.find_node_at(pos) {
                return Some(found);
            }
        }

        Some(self)
    }

    /// Recursively collects all nodes intersecting the given range.
    pub fn find_nodes_in_range(&self, range: TextRange) -> Vec<&SyntaxNode> {
        let mut results = Vec::new();
        if self.range.intersects(&range) {
            results.push(self);
            for child in &self.children {
                results.extend(child.find_nodes_in_range(range));
            }
        }
        results
    }

    /// Performs a depth-first traversal over all nodes in the subtree.
    pub fn walk<F>(&self, mut callback: F)
    where
        F: FnMut(&SyntaxNode),
    {
        self.walk_internal(&mut callback);
    }

    fn walk_internal<F>(&self, callback: &mut F)
    where
        F: FnMut(&SyntaxNode),
    {
        callback(self);
        for child in &self.children {
            child.walk_internal(callback);
        }
    }

    /// Returns named child nodes.
    pub fn named_children(&self) -> Vec<&SyntaxNode> {
        self.children.iter().filter(|c| c.is_named).collect()
    }

    /// Finds a direct child matching a given name.
    pub fn child_by_name(&self, name: &str) -> Option<&SyntaxNode> {
        self.children.iter().find(|c| c.name == name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_syntax_node_traversal() {
        let leaf = SyntaxNode::new(
            NodeKind::Key,
            "key",
            TextRange::new(Position::new(0, 0, 0), Position::new(0, 3, 3)),
            vec![],
        );

        let root = SyntaxNode::new(
            NodeKind::Document,
            "document",
            TextRange::new(Position::new(0, 0, 0), Position::new(1, 0, 10)),
            vec![leaf],
        );

        let source = "key = val\n";
        assert_eq!(root.children[0].text(source), "key");

        let found = root.find_node_at(Position::new(0, 1, 1));
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "key");
    }
}
