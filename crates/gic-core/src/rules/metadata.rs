use super::{
    category::RuleCategory, documentation::RuleDocumentation, priority::RulePriority,
    severity::RuleSeverity, tags::RuleTags,
};

/// Comprehensive metadata describing a single rule.
#[derive(Debug, Clone)]
pub struct RuleMetadata {
    /// A unique identifier for the rule (e.g., "K8S_1001").
    pub id: String,
    /// A human-readable name for the rule.
    pub name: String,
    /// A short description of what the rule checks.
    pub short_description: String,
    /// A detailed explanation of the rule.
    pub long_description: String,
    /// The language or engine this rule applies to (e.g., "kubernetes", "docker").
    pub language: String,
    /// The category of the rule.
    pub category: RuleCategory,
    /// The default severity if this rule is violated.
    pub severity: RuleSeverity,
    /// The execution priority of the rule.
    pub priority: RulePriority,
    /// The version of the rule, useful for backwards compatibility.
    pub version: String,
    /// The author or organization that created the rule.
    pub author: String,
    /// Documentation details and examples.
    pub documentation: RuleDocumentation,
    /// Categorization tags for filtering.
    pub tags: RuleTags,
    /// Whether the rule is enabled by default.
    pub enabled: bool,
}
