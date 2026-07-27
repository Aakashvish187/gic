//! Unit tests for Unicode, UTF-8, and Emoji handling in YAML Intelligence Engine.

use crate::yaml::engine::YamlEngine;
use crate::yaml::parser::YamlParser;

#[test]
fn test_unicode_and_emoji_parsing() {
    let source = "title: 🚀 Infrastructure Editor GIC\ndescription: ⚡ High-performance YAML Intelligence\nauthor: ⚡ GIC Developer Team 🦀";
    let mut parser = YamlParser::new();
    let ast = parser.parse(source).unwrap();

    assert_eq!(ast.documents.len(), 1);
    assert!(ast.documents[0].root.is_some());
}

#[test]
fn test_unicode_validation_and_formatting() {
    let source = "service: 📦 Kubernetes Controller\n\tport: 443";
    let engine = YamlEngine::default();
    let (internal_diags, _) = engine.validate(source);

    assert!(!internal_diags.is_empty());
    let formatted = engine.format(source).unwrap();
    assert!(!formatted.contains('\t'));
}
