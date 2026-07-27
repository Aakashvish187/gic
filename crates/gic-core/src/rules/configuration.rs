use super::{priority::RulePriority, profile::RuleProfile, severity::RuleSeverity};
use std::collections::HashMap;

/// Configuration overrides for a specific rule.
#[derive(Debug, Clone, Default)]
pub struct RuleConfiguration {
    /// Whether the rule is explicitly enabled or disabled.
    pub enabled: Option<bool>,
    /// Override for the rule's severity.
    pub severity: Option<RuleSeverity>,
    /// Override for the rule's priority.
    pub priority: Option<RulePriority>,
    /// Additional custom string-based settings.
    pub settings: HashMap<String, String>,
}

/// Settings applied at the workspace or project level.
#[derive(Debug, Clone, Default)]
pub struct WorkspaceSettings {
    /// The base profile to use.
    pub profile: RuleProfile,
    /// Specific rule overrides, keyed by the Rule ID.
    pub rule_overrides: HashMap<String, RuleConfiguration>,
}

impl WorkspaceSettings {
    /// Creates a new configuration with the default profile.
    pub fn new() -> Self {
        Self {
            profile: RuleProfile::default(),
            rule_overrides: HashMap::new(),
        }
    }

    /// Creates a new configuration with a specific profile.
    pub fn with_profile(profile: RuleProfile) -> Self {
        Self {
            profile,
            rule_overrides: HashMap::new(),
        }
    }

    /// Adds an override for a specific rule.
    pub fn override_rule(mut self, rule_id: impl Into<String>, config: RuleConfiguration) -> Self {
        self.rule_overrides.insert(rule_id.into(), config);
        self
    }

    /// Checks if a rule is explicitly disabled.
    pub fn is_rule_disabled(&self, rule_id: &str) -> bool {
        if let Some(config) = self.rule_overrides.get(rule_id) {
            if let Some(enabled) = config.enabled {
                return !enabled;
            }
        }
        false
    }
}
