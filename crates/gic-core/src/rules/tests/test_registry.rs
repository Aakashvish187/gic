use crate::rules::*;
use std::sync::Arc;

pub(crate) struct MockRule {
    pub metadata: RuleMetadata,
}

impl MockRule {
    pub fn new(id: &str, priority: RulePriority) -> Self {
        Self {
            metadata: RuleMetadata {
                id: id.to_string(),
                name: format!("Test Rule {}", id),
                short_description: "Short desc".into(),
                long_description: "Long desc".into(),
                language: "test".into(),
                category: RuleCategory::Support,
                severity: RuleSeverity::Information,
                priority,
                version: "1.0".into(),
                author: "Test".into(),
                documentation: RuleDocumentation::default(),
                tags: RuleTags::default(),
                enabled: true,
            },
        }
    }
}

impl Rule for MockRule {
    fn metadata(&self) -> &RuleMetadata {
        &self.metadata
    }

    fn evaluate(&self, _ctx: &dyn RuleContext) -> Result<Vec<Box<dyn RuleAction>>> {
        Ok(vec![])
    }
}

#[test]
fn test_registry_registration() {
    let registry = RuleRegistry::new();
    let rule = Arc::new(MockRule::new("TEST_001", RulePriority::Medium));

    assert!(registry.register(rule.clone()).is_ok());
    assert_eq!(registry.count(), 1);

    // Duplicate registration should fail
    assert!(registry.register(rule).is_err());
}

#[test]
fn test_registry_retrieval() {
    let registry = RuleRegistry::new();
    let rule = Arc::new(MockRule::new("TEST_002", RulePriority::High));
    registry.register(rule).unwrap();

    assert!(registry.get("TEST_002").is_ok());
    assert!(registry.get("NON_EXISTENT").is_err());
}
