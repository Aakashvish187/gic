//! Validation Rule abstraction, metadata, categories, priorities, and configuration.

use crate::diagnostics::diagnostic::Diagnostic;
use crate::diagnostics::errors::DiagnosticResult;
use crate::diagnostics::severity::DiagnosticLevel;
use crate::diagnostics::validator::ValidationContext;
use crate::parser::language::LanguageId;
use crate::parser::tree::SyntaxTree;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// Categorizes validation rules according to domain purpose.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum RuleCategory {
    /// Pure syntax structure and grammar validation.
    Syntax,
    /// Security flaws, permissions, credentials, and vulnerability checks.
    Security,
    /// Resource usage, efficiency, and speed bottlenecks.
    Performance,
    /// Formatting, naming conventions, and readability rules.
    Style,
    /// Infrastructure and language best practice conventions.
    BestPractice,
    /// User-defined or custom plugin validation category.
    Custom(String),
}

impl fmt::Display for RuleCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RuleCategory::Syntax => write!(f, "Syntax"),
            RuleCategory::Security => write!(f, "Security"),
            RuleCategory::Performance => write!(f, "Performance"),
            RuleCategory::Style => write!(f, "Style"),
            RuleCategory::BestPractice => write!(f, "BestPractice"),
            RuleCategory::Custom(name) => write!(f, "Custom({})", name),
        }
    }
}

/// Execution priority level for ordering rule evaluations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum RulePriority {
    /// Informational or low priority checks.
    Low = 0,
    /// Standard rule execution priority.
    Medium = 1,
    /// High priority rule evaluated before standard rules.
    High = 2,
    /// Critical priority rule (e.g. core syntax errors) evaluated first.
    Critical = 3,
}

/// Comprehensive metadata describing a validation rule.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuleMetadata {
    /// Unique rule identifier (e.g. "GIC-SEC-001").
    pub id: String,
    /// Human-readable rule name.
    pub name: String,
    /// Detailed description of what the rule checks and why.
    pub description: String,
    /// Rule domain category.
    pub category: RuleCategory,
    /// Rule priority.
    pub priority: RulePriority,
    /// Default diagnostic level produced when rule fails.
    pub default_level: DiagnosticLevel,
    /// Target languages supported by this rule (empty means all).
    pub supported_languages: Vec<LanguageId>,
    /// Optional URL for online documentation.
    pub documentation_url: Option<String>,
}

impl RuleMetadata {
    /// Creates a new `RuleMetadata` instance.
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        category: RuleCategory,
        default_level: DiagnosticLevel,
    ) -> Self {
        let name_str = name.into();
        Self {
            id: id.into(),
            name: name_str.clone(),
            description: String::new(),
            category,
            priority: RulePriority::Medium,
            default_level,
            supported_languages: Vec::new(),
            documentation_url: None,
        }
    }

    /// Sets rule description.
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    /// Sets rule priority.
    pub fn with_priority(mut self, priority: RulePriority) -> Self {
        self.priority = priority;
        self
    }

    /// Restricts rule to specific languages.
    pub fn with_languages(mut self, languages: Vec<LanguageId>) -> Self {
        self.supported_languages = languages;
        self
    }

    /// Sets documentation URL.
    pub fn with_doc_url(mut self, url: impl Into<String>) -> Self {
        self.documentation_url = Some(url.into());
        self
    }
}

/// User or system runtime configuration for a specific rule.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuleConfig {
    /// Whether the rule is actively enabled.
    pub enabled: bool,
    /// Optional priority override.
    pub priority_override: Option<RulePriority>,
    /// Optional diagnostic level override.
    pub severity_override: Option<DiagnosticLevel>,
    /// Key-value parameters for rule customization.
    pub options: HashMap<String, String>,
}

impl Default for RuleConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            priority_override: None,
            severity_override: None,
            options: HashMap::new(),
        }
    }
}

impl RuleConfig {
    /// Creates an enabled configuration with default parameters.
    pub fn enabled() -> Self {
        Self::default()
    }

    /// Creates a disabled rule configuration.
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            ..Self::default()
        }
    }

    /// Returns the effective severity level given default rule level.
    pub fn effective_severity(&self, default: DiagnosticLevel) -> DiagnosticLevel {
        self.severity_override.unwrap_or(default)
    }

    /// Returns the effective priority level given default rule priority.
    pub fn effective_priority(&self, default: RulePriority) -> RulePriority {
        self.priority_override.unwrap_or(default)
    }
}

/// Trait implemented by all validation rules in the GIC system.
pub trait Rule: Send + Sync {
    /// Returns reference to rule metadata.
    fn metadata(&self) -> &RuleMetadata;

    /// Evaluates the rule against a parsed syntax tree and validation context.
    fn evaluate(
        &self,
        tree: &SyntaxTree,
        ctx: &ValidationContext,
    ) -> DiagnosticResult<Vec<Diagnostic>>;

    /// Returns `true` if rule applies to the specified language.
    fn supports_language(&self, lang: LanguageId) -> bool {
        let langs = &self.metadata().supported_languages;
        langs.is_empty() || langs.contains(&lang)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rule_metadata_and_config() {
        let meta = RuleMetadata::new(
            "GIC-001",
            "NoTabs",
            RuleCategory::Style,
            DiagnosticLevel::Warning,
        )
        .with_priority(RulePriority::High)
        .with_languages(vec![LanguageId::Yaml, LanguageId::Dockerfile]);

        assert_eq!(meta.name, "NoTabs");
        assert_eq!(meta.priority, RulePriority::High);

        let mut config = RuleConfig::enabled();
        config.severity_override = Some(DiagnosticLevel::Error);

        assert_eq!(
            config.effective_severity(meta.default_level),
            DiagnosticLevel::Error
        );
    }
}
