//! Diagnostic severity levels and priority classifications.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Diagnostic levels supported by the GIC Validation & Diagnostics Engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum DiagnosticLevel {
    /// Severe syntax, schema, or structural failure requiring fix before deployment.
    Error,
    /// Warning indicating potential bug, missing field, or deprecated usage.
    Warning,
    /// Informational message regarding code organization or metadata.
    Information,
    /// Editor suggestion, code hint, or subtle improvement idea.
    Hint,
    /// Future compatibility or deprecation notice for upcoming API versions.
    Future,
    /// Security vulnerability, permission flaw, or credentials exposure.
    Security,
    /// Performance bottleneck, redundant execution, or resource leak.
    Performance,
    /// Code style, formatting, or naming convention issue.
    Style,
    /// Infrastructure or software development best practice recommendation.
    BestPractice,
}

impl DiagnosticLevel {
    /// Returns `true` if this diagnostic level indicates a critical blocking issue.
    pub fn is_critical(&self) -> bool {
        matches!(self, DiagnosticLevel::Error | DiagnosticLevel::Security)
    }

    /// Returns a short visual tag / label for UI formatting.
    pub fn tag(&self) -> &'static str {
        match self {
            DiagnosticLevel::Error => "ERROR",
            DiagnosticLevel::Warning => "WARN",
            DiagnosticLevel::Information => "INFO",
            DiagnosticLevel::Hint => "HINT",
            DiagnosticLevel::Future => "FUTURE",
            DiagnosticLevel::Security => "SEC",
            DiagnosticLevel::Performance => "PERF",
            DiagnosticLevel::Style => "STYLE",
            DiagnosticLevel::BestPractice => "BEST",
        }
    }

    /// Returns a single unicode indicator symbol for CLI / terminal display.
    pub fn symbol(&self) -> &'static str {
        match self {
            DiagnosticLevel::Error => "✖",
            DiagnosticLevel::Warning => "⚠",
            DiagnosticLevel::Information => "ℹ",
            DiagnosticLevel::Hint => "💡",
            DiagnosticLevel::Future => "🔮",
            DiagnosticLevel::Security => "🛡",
            DiagnosticLevel::Performance => "⚡",
            DiagnosticLevel::Style => "🎨",
            DiagnosticLevel::BestPractice => "⭐",
        }
    }
}

impl fmt::Display for DiagnosticLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.tag())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diagnostic_level_critical() {
        assert!(DiagnosticLevel::Error.is_critical());
        assert!(DiagnosticLevel::Security.is_critical());
        assert!(!DiagnosticLevel::Warning.is_critical());
        assert!(!DiagnosticLevel::Style.is_critical());
    }

    #[test]
    fn test_diagnostic_level_display_and_tag() {
        assert_eq!(DiagnosticLevel::Error.tag(), "ERROR");
        assert_eq!(DiagnosticLevel::Performance.tag(), "PERF");
        assert_eq!(format!("{}", DiagnosticLevel::Security), "SEC");
        assert_eq!(DiagnosticLevel::Hint.symbol(), "💡");
    }
}
