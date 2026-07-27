//! Diagnostic and validation engine error definitions.

use std::fmt;

/// Errors that can occur during diagnostic generation, validation, or rule execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiagnosticError {
    /// Validator for a specific language or type was not found.
    ValidatorNotFound(String),
    /// Specified validation rule was not found in the registry.
    RuleNotFound(String),
    /// Invalid rule configuration or parameters.
    InvalidRuleConfig(String),
    /// Validation operation was cancelled before completion.
    ValidationCancelled,
    /// Diagnostic cache error or inconsistency.
    CacheError(String),
    /// Execution error during rule evaluation.
    ExecutionError(String),
    /// Generic system or unexpected error.
    SystemError(String),
}

impl fmt::Display for DiagnosticError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DiagnosticError::ValidatorNotFound(lang) => {
                write!(f, "Validator not found for language/type: '{}'", lang)
            }
            DiagnosticError::RuleNotFound(rule) => {
                write!(f, "Validation rule not found: '{}'", rule)
            }
            DiagnosticError::InvalidRuleConfig(msg) => {
                write!(f, "Invalid rule configuration: {}", msg)
            }
            DiagnosticError::ValidationCancelled => {
                write!(f, "Validation operation was cancelled")
            }
            DiagnosticError::CacheError(msg) => {
                write!(f, "Diagnostic cache error: {}", msg)
            }
            DiagnosticError::ExecutionError(msg) => {
                write!(f, "Validation execution error: {}", msg)
            }
            DiagnosticError::SystemError(msg) => {
                write!(f, "Diagnostic system error: {}", msg)
            }
        }
    }
}

impl std::error::Error for DiagnosticError {}

/// Type alias for diagnostic operations.
pub type DiagnosticResult<T> = Result<T, DiagnosticError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = DiagnosticError::ValidatorNotFound("yaml".to_string());
        assert_eq!(
            err.to_string(),
            "Validator not found for language/type: 'yaml'"
        );

        let err2 = DiagnosticError::ValidationCancelled;
        assert_eq!(err2.to_string(), "Validation operation was cancelled");
    }
}
