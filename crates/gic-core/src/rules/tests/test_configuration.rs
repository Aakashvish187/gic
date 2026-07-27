use super::test_registry::MockRule;
use crate::rules::*;
use std::sync::Arc;

#[test]
fn test_workspace_settings_override() {
    let mut settings = WorkspaceSettings::new();

    // By default, it's not disabled unless manually overridden
    assert!(!settings.is_rule_disabled("TEST_001"));

    settings = settings.override_rule(
        "TEST_001",
        RuleConfiguration {
            enabled: Some(false),
            ..Default::default()
        },
    );

    assert!(settings.is_rule_disabled("TEST_001"));
}

#[test]
fn test_scheduler_priority_sorting() {
    let registry = Arc::new(RuleRegistry::new());

    registry
        .register(Arc::new(MockRule::new("LOW", RulePriority::Low)))
        .unwrap();
    registry
        .register(Arc::new(MockRule::new("HIGH", RulePriority::High)))
        .unwrap();
    registry
        .register(Arc::new(MockRule::new("MEDIUM", RulePriority::Medium)))
        .unwrap();

    let scheduler = RuleScheduler::new(registry);
    let settings = WorkspaceSettings::new();
    let active = scheduler.get_active_rules(&settings);

    assert_eq!(active.len(), 3);
    assert_eq!(active[0].metadata().id, "HIGH");
    assert_eq!(active[1].metadata().id, "MEDIUM");
    assert_eq!(active[2].metadata().id, "LOW");
}
