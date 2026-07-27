//! Unit tests for YAML Parser and Tokenizer.

use crate::yaml::parser::{YamlParser, YamlValue};

#[test]
fn test_parse_empty_document() {
    let mut parser = YamlParser::new();
    let ast = parser.parse("").unwrap();
    assert!(ast.is_empty());
}

#[test]
fn test_parse_simple_mapping() {
    let source = "apiVersion: v1\nkind: Pod";
    let mut parser = YamlParser::new();
    let ast = parser.parse(source).unwrap();

    assert_eq!(ast.documents.len(), 1);
    let doc = &ast.documents[0];
    assert!(doc.root.is_some());

    let root = doc.root.as_ref().unwrap();
    if let YamlValue::Mapping(ref map) = root.value {
        assert_eq!(map.pairs.len(), 2);
        assert_eq!(map.pairs[0].key.value, "apiVersion");
        assert_eq!(map.pairs[1].key.value, "kind");
    } else {
        panic!("Expected mapping root node");
    }
}

#[test]
fn test_parse_multi_document_stream() {
    let source = "---\nname: doc1\n---\nname: doc2\n...";
    let mut parser = YamlParser::new();
    let ast = parser.parse(source).unwrap();

    assert_eq!(ast.documents.len(), 2);
    assert!(ast.documents[0].has_explicit_start);
    assert!(ast.documents[1].has_explicit_start);
    assert!(ast.documents[1].has_explicit_end);
}

#[test]
fn test_parse_sequence() {
    let source = "- item1\n- item2\n- item3";
    let mut parser = YamlParser::new();
    let ast = parser.parse(source).unwrap();

    let doc = &ast.documents[0];
    let root = doc.root.as_ref().unwrap();
    if let YamlValue::Sequence(ref seq) = root.value {
        assert_eq!(seq.items.len(), 3);
    } else {
        panic!("Expected sequence root node");
    }
}
