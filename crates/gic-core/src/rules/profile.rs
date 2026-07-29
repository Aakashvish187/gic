use std::fmt;

/// Predefined configuration profiles for the Rule Engine.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum RuleProfile {
    /// Enable all rules and treat warnings as errors.
    Strict,
    /// Enable rules suitable for production environments.
    Production,
    /// Enable rules suitable for local development.
    #[default]
    Development,
    /// Focus heavily on security rules.
    Security,
    /// Focus heavily on performance optimization rules.
    Performance,
    /// Enable only critical rules.
    Minimal,
    /// A custom, user-defined profile.
    Custom(String),
}

impl fmt::Display for RuleProfile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RuleProfile::Strict => write!(f, "Strict"),
            RuleProfile::Production => write!(f, "Production"),
            RuleProfile::Development => write!(f, "Development"),
            RuleProfile::Security => write!(f, "Security"),
            RuleProfile::Performance => write!(f, "Performance"),
            RuleProfile::Minimal => write!(f, "Minimal"),
            RuleProfile::Custom(s) => write!(f, "{}", s),
        }
    }
}
