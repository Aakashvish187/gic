//! YAML Anchor Analysis and Tracking Engine.
//!
//! Extracts, indexes, and validates all YAML anchor definitions (`&anchor_name`)
//! across documents.

use std::collections::HashMap;

use crate::yaml::parser::{Span, YamlAST, YamlNode, YamlValue};

/// Information about a defined anchor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnchorInfo {
    /// Anchor identifier name (without leading `&`).
    pub name: String,
    /// Unique AST node ID to which the anchor is attached.
    pub node_id: usize,
    /// Span of the anchor token.
    pub token_span: Span,
    /// Span of the anchored node.
    pub node_span: Span,
}

/// Registry storing all anchors defined in a document stream.
#[derive(Debug, Clone, Default)]
pub struct AnchorRegistry {
    anchors: HashMap<String, AnchorInfo>,
}

impl AnchorRegistry {
    /// Creates a new empty AnchorRegistry.
    pub fn new() -> Self {
        Self {
            anchors: HashMap::new(),
        }
    }

    /// Registers a new anchor definition. Returns previous anchor if re-defined.
    pub fn register(&mut self, info: AnchorInfo) -> Option<AnchorInfo> {
        self.anchors.insert(info.name.clone(), info)
    }

    /// Retrieves anchor information by name.
    pub fn get(&self, name: &str) -> Option<&AnchorInfo> {
        self.anchors.get(name)
    }

    /// Returns true if an anchor with the given name exists.
    pub fn contains(&self, name: &str) -> bool {
        self.anchors.contains_key(name)
    }

    /// Iterator over all registered anchors.
    pub fn iter(&self) -> impl Iterator<Item = (&String, &AnchorInfo)> {
        self.anchors.iter()
    }
}

/// Extractor for anchor definitions in a `YamlAST`.
#[derive(Debug, Clone, Default)]
pub struct AnchorExtractor;

impl AnchorExtractor {
    /// Creates a new AnchorExtractor.
    pub fn new() -> Self {
        Self
    }

    /// Extracts all anchor definitions from a `YamlAST` into an `AnchorRegistry`.
    pub fn extract(&self, ast: &YamlAST) -> AnchorRegistry {
        let mut registry = AnchorRegistry::new();
        for doc in &ast.documents {
            if let Some(ref root) = doc.root {
                self.extract_from_node(root, &mut registry);
            }
        }
        registry
    }

    fn extract_from_node(&self, node: &YamlNode, registry: &mut AnchorRegistry) {
        if let Some(ref anchor) = node.anchor {
            registry.register(AnchorInfo {
                name: anchor.name.clone(),
                node_id: node.id,
                token_span: anchor.span,
                node_span: node.span,
            });
        }

        match &node.value {
            YamlValue::Mapping(mapping) => {
                for pair in &mapping.pairs {
                    self.extract_from_node(&pair.value, registry);
                }
            }
            YamlValue::Sequence(seq) => {
                for item in &seq.items {
                    self.extract_from_node(item, registry);
                }
            }
            _ => {}
        }
    }
}
