//! Unit tests for YAML Code Folding Engine.

use crate::yaml::folding::{FoldingKind, YamlFoldingEngine};
use crate::yaml::parser::YamlParser;

#[test]
fn test_folding_ranges_calculation() {
    let source = "parent:\n  child1: v1\n  child2: v2\n  child3: v3";
    let mut parser = YamlParser::new();
    let ast = parser.parse(source).unwrap();

    let folding_engine = YamlFoldingEngine::new();
    let ranges = folding_engine.compute_folding_ranges(&ast);

    assert!(!ranges.is_empty());
    let mapping_fold = ranges.iter().find(|r| r.kind == FoldingKind::Mapping);
    assert!(mapping_fold.is_some());
    assert_eq!(mapping_fold.unwrap().start_line, 1);
    assert_eq!(mapping_fold.unwrap().end_line, 4);
}

#[test]
fn test_comment_block_folding() {
    let source = "# Comment line 1\n# Comment line 2\n# Comment line 3\nkey: val";
    let mut parser = YamlParser::new();
    let ast = parser.parse(source).unwrap();

    let folding_engine = YamlFoldingEngine::new();
    let ranges = folding_engine.compute_folding_ranges(&ast);

    let comment_fold = ranges.iter().find(|r| r.kind == FoldingKind::Comment);
    assert!(comment_fold.is_some());
    assert_eq!(comment_fold.unwrap().start_line, 1);
    assert_eq!(comment_fold.unwrap().end_line, 3);
}
