use std::fmt;

/// Errors that can occur within the Universal Rule Engine.
#[derive(Debug)]
pub enum RuleEngineError {
    /// A rule with the given ID is already registered.
    DuplicateRule(String),
    /// A requested rule was not found in the registry.
    RuleNotFound(String),
    /// An error occurred during rule evaluation.
    EvaluationError { rule_id: String, message: String },
    /// An error occurred while parsing or loading a rule configuration.
    ConfigurationError(String),
    /// An error related to rule caching.
    CacheError(String),
    /// General or unknown engine error.
    InternalError(String),
}

impl fmt::Display for RuleEngineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RuleEngineError::DuplicateRule(id) => write!(f, "Rule already registered: {}", id),
            RuleEngineError::RuleNotFound(id) => write!(f, "Rule not found: {}", id),
            RuleEngineError::EvaluationError { rule_id, message } => {
                write!(f, "Evaluation error in rule '{}': {}", rule_id, message)
            }
            RuleEngineError::ConfigurationError(msg) => write!(f, "Configuration error: {}", msg),
            RuleEngineError::CacheError(msg) => write!(f, "Cache error: {}", msg),
            RuleEngineError::InternalError(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl std::error::Error for RuleEngineError {}

/// A specialized Result type for the Universal Rule Engine.
pub type Result<T> = std::result::Result<T, RuleEngineError>;
