//! Performance and stress tests for the Diagnostics & Validation Engine.

use crate::diagnostics::engine::ValidationEngine;
use crate::parser::language::LanguageId;
use crate::parser::node::{NodeKind, SyntaxNode};
use crate::parser::position::{Position, TextRange};
use crate::parser::tree::SyntaxTree;
use std::time::Instant;

#[test]
fn test_large_document_validation_performance() {
    let engine = ValidationEngine::new();

    // Create a large 10,000 line mock source text
    let mut large_text = String::with_capacity(500_000);
    for i in 1..=10_000 {
        large_text.push_str(&format!("key_{}: value_{}\n", i, i));
    }

    let root = SyntaxNode::new(
        NodeKind::Document,
        "document",
        TextRange::empty(Position::zero()),
        Vec::new(),
    );
    let tree = SyntaxTree::new(
        LanguageId::Yaml,
        root,
        Vec::new(),
        Vec::new(),
        777111,
        large_text.len(),
    );

    let start = Instant::now();
    let diags = engine
        .validate("large_file.yaml", &tree, &large_text, None)
        .unwrap();
    let elapsed = start.elapsed();

    assert!(diags.is_empty());
    // Validation should complete under 500ms for 10,000 lines
    assert!(
        elapsed.as_millis() < 500,
        "Large document validation took too long: {} ms",
        elapsed.as_millis()
    );
}
