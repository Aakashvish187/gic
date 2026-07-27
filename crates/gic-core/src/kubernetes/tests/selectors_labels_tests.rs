//! Unit tests for Label Syntax Validation and LabelSelector Matching.

use std::collections::HashMap;

use crate::kubernetes::labels::LabelValidator;
use crate::kubernetes::selectors::{LabelSelector, SelectorOperator, SelectorRequirement};

#[test]
fn test_label_syntax_validation() {
    let validator = LabelValidator::new();

    let mut labels = HashMap::new();
    labels.insert("app.kubernetes.io/name".to_string(), "backend".to_string());
    labels.insert("invalid key!".to_string(), "val".to_string());

    let issues = validator.validate(&labels);
    assert_eq!(issues.len(), 1);
    assert!(issues[0].message.contains("Invalid label key"));
}

#[test]
fn test_selector_matching() {
    let mut match_labels = HashMap::new();
    match_labels.insert("app".to_string(), "web".to_string());

    let mut selector = LabelSelector::from_match_labels(match_labels);
    selector.match_expressions.push(SelectorRequirement {
        key: "env".to_string(),
        operator: SelectorOperator::In,
        values: vec!["prod".to_string(), "staging".to_string()],
    });

    let mut target_labels = HashMap::new();
    target_labels.insert("app".to_string(), "web".to_string());
    target_labels.insert("env".to_string(), "prod".to_string());

    assert!(selector.matches(&target_labels));

    target_labels.insert("env".to_string(), "dev".to_string());
    assert!(!selector.matches(&target_labels));
}
