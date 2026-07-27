use std::fmt;

/// Defines the severity level of a rule violation.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RuleSeverity {
    /// Informational level used by internal support rules.
    Support,
    /// Suggestion for alternative approaches.
    Suggestion,
    /// A hint for potential improvements.
    Hint,
    /// General information.
    Information,
    /// A warning about a potential issue.
    Warning,
    /// A clear error that needs fixing.
    Error,
    /// A critical violation that must be addressed immediately.
    Critical,
    /// A custom severity level.
    Custom(String),
}

impl fmt::Display for RuleSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RuleSeverity::Support => write!(f, "Support"),
            RuleSeverity::Suggestion => write!(f, "Suggestion"),
            RuleSeverity::Hint => write!(f, "Hint"),
            RuleSeverity::Information => write!(f, "Information"),
            RuleSeverity::Warning => write!(f, "Warning"),
            RuleSeverity::Error => write!(f, "Error"),
            RuleSeverity::Critical => write!(f, "Critical"),
            RuleSeverity::Custom(s) => write!(f, "{}", s),
        }
    }
}
