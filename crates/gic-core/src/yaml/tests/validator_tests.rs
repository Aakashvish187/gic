//! Unit tests for YAML Validation Engine.

use crate::yaml::validator::{YamlSeverity, YamlValidator};

#[test]
fn test_detect_tabs_in_indentation() {
    let source = "server:\n\thost: localhost";
    let validator = YamlValidator::default();
    let diags = validator.validate_source(source);

    assert!(!diags.is_empty());
    let tab_diag = diags.iter().find(|d| d.rule_id == "yaml-no-tabs");
    assert!(tab_diag.is_some());
    assert_eq!(tab_diag.unwrap().severity, YamlSeverity::Error);
    assert!(tab_diag.unwrap().quick_fix.is_some());
}

#[test]
fn test_detect_duplicate_mapping_keys() {
    let source = "key1: value1\nkey1: value2";
    let validator = YamlValidator::default();
    let diags = validator.validate_source(source);

    let dup_diag = diags.iter().find(|d| d.rule_id == "yaml-duplicate-key");
    assert!(dup_diag.is_some());
    assert_eq!(dup_diag.unwrap().severity, YamlSeverity::Error);
}

#[test]
fn test_valid_yaml_source_produces_no_errors() {
    let source = "name: test-app\nversion: 1.0.0\nenv:\n  - production";
    let validator = YamlValidator::default();
    let diags = validator.validate_source(source);

    let errors: Vec<_> = diags
        .into_iter()
        .filter(|d| d.severity == YamlSeverity::Error)
        .collect();
    assert!(errors.is_empty());
}
