//! Unit tests for YAML Anchors and Aliases resolution.

use crate::yaml::aliases::AliasResolver;
use crate::yaml::anchors::AnchorExtractor;
use crate::yaml::parser::YamlParser;

#[test]
fn test_anchor_extraction_and_alias_resolution() {
    let source = "default_env: &default_anchor dev\napp_env: *default_anchor";
    let mut parser = YamlParser::new();
    let ast = parser.parse(source).unwrap();

    let extractor = AnchorExtractor::new();
    let registry = extractor.extract(&ast);
    assert!(registry.contains("default_anchor"));

    let resolver = AliasResolver::new();
    let issues = resolver.resolve(&ast, &registry);
    assert!(issues.is_empty());
}

#[test]
fn test_unresolved_alias_detection() {
    let source = "app_env: *missing_anchor";
    let mut parser = YamlParser::new();
    let ast = parser.parse(source).unwrap();

    let extractor = AnchorExtractor::new();
    let registry = extractor.extract(&ast);
    let resolver = AliasResolver::new();
    let issues = resolver.resolve(&ast, &registry);

    assert_eq!(issues.len(), 1);
    assert_eq!(
        issues[0].kind,
        crate::yaml::aliases::AliasIssueKind::UnresolvedAlias
    );
}
