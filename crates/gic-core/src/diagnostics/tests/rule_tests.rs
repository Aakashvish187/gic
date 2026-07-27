//! Unit tests for Rule Metadata, Rule Categories, Rule Priorities, and Rule Configurations.

use crate::diagnostics::diagnostic::Diagnostic;
use crate::diagnostics::errors::DiagnosticResult;
use crate::diagnostics::registry::RuleRegistry;
use crate::diagnostics::rule::{Rule, RuleCategory, RuleConfig, RuleMetadata, RulePriority};
use crate::diagnostics::severity::DiagnosticLevel;
use crate::diagnostics::validator::ValidationContext;
use crate::parser::language::LanguageId;
use crate::parser::tree::SyntaxTree;

struct DummyRule {
    meta: RuleMetadata,
}

impl Rule for DummyRule {
    fn metadata(&self) -> &RuleMetadata {
        &self.meta
    }

    fn evaluate(
        &self,
        _tree: &SyntaxTree,
        _ctx: &ValidationContext,
    ) -> DiagnosticResult<Vec<Diagnostic>> {
        Ok(Vec::new())
    }
}

#[test]
fn test_rule_priority_sorting() {
    let mut reg = RuleRegistry::new();

    let rule_low = DummyRule {
        meta: RuleMetadata::new(
            "R-LOW",
            "LowRule",
            RuleCategory::Style,
            DiagnosticLevel::Style,
        )
        .with_priority(RulePriority::Low),
    };
    let rule_crit = DummyRule {
        meta: RuleMetadata::new(
            "R-CRIT",
            "CritRule",
            RuleCategory::Security,
            DiagnosticLevel::Security,
        )
        .with_priority(RulePriority::Critical),
    };
    let rule_med = DummyRule {
        meta: RuleMetadata::new(
            "R-MED",
            "MedRule",
            RuleCategory::BestPractice,
            DiagnosticLevel::Warning,
        )
        .with_priority(RulePriority::Medium),
    };

    reg.register(rule_low);
    reg.register(rule_crit);
    reg.register(rule_med);

    let active_rules = reg.get_rules_for_language(LanguageId::Yaml);
    assert_eq!(active_rules.len(), 3);
    assert_eq!(active_rules[0].metadata().id, "R-CRIT");
    assert_eq!(active_rules[1].metadata().id, "R-MED");
    assert_eq!(active_rules[2].metadata().id, "R-LOW");
}

#[test]
fn test_rule_enable_disable_and_category() {
    let mut reg = RuleRegistry::new();

    let rule_sec = DummyRule {
        meta: RuleMetadata::new(
            "SEC-01",
            "SecRule",
            RuleCategory::Security,
            DiagnosticLevel::Security,
        ),
    };
    let rule_style = DummyRule {
        meta: RuleMetadata::new(
            "STYLE-01",
            "StyleRule",
            RuleCategory::Style,
            DiagnosticLevel::Hint,
        ),
    };

    reg.register(rule_sec);
    reg.register(rule_style);

    let sec_rules = reg.get_rules_by_category(&RuleCategory::Security);
    assert_eq!(sec_rules.len(), 1);
    assert_eq!(sec_rules[0].metadata().id, "SEC-01");

    reg.set_enabled("SEC-01", false).unwrap();
    let yaml_rules = reg.get_rules_for_language(LanguageId::Yaml);
    assert_eq!(yaml_rules.len(), 1);
    assert_eq!(yaml_rules[0].metadata().id, "STYLE-01");
}

#[test]
fn test_rule_config_overrides() {
    let meta = RuleMetadata::new(
        "R-01",
        "Rule1",
        RuleCategory::Performance,
        DiagnosticLevel::Information,
    )
    .with_priority(RulePriority::Low);

    let mut cfg = RuleConfig::enabled();
    cfg.severity_override = Some(DiagnosticLevel::Error);
    cfg.priority_override = Some(DiagnosticLevel::Security.into_priority_override_test()); // test helper logic

    assert_eq!(
        cfg.effective_severity(meta.default_level),
        DiagnosticLevel::Error
    );
}

trait PriorityTestExt {
    fn into_priority_override_test(&self) -> RulePriority;
}

impl PriorityTestExt for DiagnosticLevel {
    fn into_priority_override_test(&self) -> RulePriority {
        RulePriority::High
    }
}
