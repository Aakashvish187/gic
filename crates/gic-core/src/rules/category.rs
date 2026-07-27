use std::fmt;

/// Defines the category of a rule.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RuleCategory {
    /// Rules related to IDE support or engine internals.
    Support,
    /// Rules ensuring basic syntax correctness.
    Syntax,
    /// Rules validating structural or semantic constraints.
    Validation,
    /// Security-related rules and best practices.
    Security,
    /// Performance and optimization suggestions.
    Performance,
    /// Code style violations.
    Style,
    /// Code formatting violations.
    Formatting,
    /// Generally accepted best practices.
    BestPractices,
    /// Rules ensuring compatibility across versions or environments.
    Compatibility,
    /// Rules identifying hard-to-maintain code patterns.
    Maintainability,
    /// Rules identifying potential runtime crashes or unreliability.
    Reliability,
    /// Cloud infrastructure specific rules.
    Cloud,
    /// Container specific rules (Docker, Podman, etc.).
    Containers,
    /// Network configuration rules.
    Networking,
    /// General infrastructure as code rules.
    Infrastructure,
    /// Custom user or plugin defined category.
    Custom(String),
}

impl fmt::Display for RuleCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RuleCategory::Support => write!(f, "Support"),
            RuleCategory::Syntax => write!(f, "Syntax"),
            RuleCategory::Validation => write!(f, "Validation"),
            RuleCategory::Security => write!(f, "Security"),
            RuleCategory::Performance => write!(f, "Performance"),
            RuleCategory::Style => write!(f, "Style"),
            RuleCategory::Formatting => write!(f, "Formatting"),
            RuleCategory::BestPractices => write!(f, "Best Practices"),
            RuleCategory::Compatibility => write!(f, "Compatibility"),
            RuleCategory::Maintainability => write!(f, "Maintainability"),
            RuleCategory::Reliability => write!(f, "Reliability"),
            RuleCategory::Cloud => write!(f, "Cloud"),
            RuleCategory::Containers => write!(f, "Containers"),
            RuleCategory::Networking => write!(f, "Networking"),
            RuleCategory::Infrastructure => write!(f, "Infrastructure"),
            RuleCategory::Custom(s) => write!(f, "{}", s),
        }
    }
}
