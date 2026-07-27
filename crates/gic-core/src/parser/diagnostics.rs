//! Diagnostic severity and parse diagnostic reports for syntax error handling.

use crate::parser::position::TextRange;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Severity levels for syntax diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum DiagnosticSeverity {
    /// Severe syntax or parse error.
    Error,
    /// Warning indicating potential syntax ambiguity or soft error.
    Warning,
    /// Informational note regarding code structure.
    Information,
    /// Editor suggestion or hint.
    Hint,
}

impl fmt::Display for DiagnosticSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DiagnosticSeverity::Error => write!(f, "ERROR"),
            DiagnosticSeverity::Warning => write!(f, "WARN"),
            DiagnosticSeverity::Information => write!(f, "INFO"),
            DiagnosticSeverity::Hint => write!(f, "HINT"),
        }
    }
}

/// Represents a single parse diagnostic message tied to a specific source text range.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Diagnostic {
    /// Severity level of the diagnostic.
    pub severity: DiagnosticSeverity,
    /// Location range within the source text.
    pub range: TextRange,
    /// Descriptive message explaining the syntax issue.
    pub message: String,
    /// Optional error code or identifier.
    pub code: Option<String>,
    /// Source parser identifier (e.g. "yaml-parser").
    pub source: String,
}

impl Diagnostic {
    /// Creates a new error diagnostic.
    pub fn error(range: TextRange, message: impl Into<String>, source: impl Into<String>) -> Self {
        Self {
            severity: DiagnosticSeverity::Error,
            range,
            message: message.into(),
            code: None,
            source: source.into(),
        }
    }

    /// Creates a new warning diagnostic.
    pub fn warning(
        range: TextRange,
        message: impl Into<String>,
        source: impl Into<String>,
    ) -> Self {
        Self {
            severity: DiagnosticSeverity::Warning,
            range,
            message: message.into(),
            code: None,
            source: source.into(),
        }
    }

    /// Attaches an error code to the diagnostic.
    pub fn with_code(mut self, code: impl Into<String>) -> Self {
        self.code = Some(code.into());
        self
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}] {} (at {}): {}",
            self.source, self.severity, self.range, self.message
        )
    }
}
