//! Central Diagnostic data structure representing a validation output.

use crate::diagnostics::quick_fix::QuickFix;
use crate::diagnostics::range::DiagnosticRange;
use crate::diagnostics::severity::DiagnosticLevel;
use crate::diagnostics::utils::{current_timestamp_ms, generate_diagnostic_id};
use crate::parser::language::LanguageId;
use serde::{Deserialize, Serialize};
use std::fmt;

/// The central Diagnostic object produced by the GIC Validation & Diagnostics Engine.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Diagnostic {
    /// Unique identifier for this diagnostic instance.
    pub id: String,
    /// Diagnostic severity level.
    pub severity: DiagnosticLevel,
    /// Short summary message describing the issue.
    pub message: String,
    /// Extended detailed description or rationale explaining the issue.
    pub description: Option<String>,
    /// 1-based line number for UI rendering and error reporting.
    pub line: usize,
    /// 1-based column number for UI rendering and error reporting.
    pub column: usize,
    /// Precise source code range spanned by this diagnostic.
    pub range: DiagnosticRange,
    /// Name or identifier of the rule that generated this diagnostic.
    pub rule_name: String,
    /// Target language format (e.g. Yaml, Dockerfile, Bash, etc.).
    pub language: LanguageId,
    /// List of suggested quick fixes to resolve the issue.
    pub quick_fixes: Vec<QuickFix>,
    /// Placeholder URL / link to online rule documentation.
    pub documentation_link: Option<String>,
    /// System timestamp in milliseconds when this diagnostic was generated.
    pub timestamp: u64,
}

impl Diagnostic {
    /// Creates a new `Diagnostic` object with all required fields.
    pub fn new(
        severity: DiagnosticLevel,
        message: impl Into<String>,
        range: DiagnosticRange,
        rule_name: impl Into<String>,
        language: LanguageId,
    ) -> Self {
        let msg = message.into();
        let r_name = rule_name.into();
        let line = range.start.line;
        let column = range.start.column;
        let id = generate_diagnostic_id(&r_name, line, column, &msg);
        let timestamp = current_timestamp_ms();

        Self {
            id,
            severity,
            message: msg,
            description: None,
            line,
            column,
            range,
            rule_name: r_name,
            language,
            quick_fixes: Vec::new(),
            documentation_link: None,
            timestamp,
        }
    }

    /// Attaches an extended description.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Attaches quick fix options.
    pub fn with_quick_fixes(mut self, fixes: Vec<QuickFix>) -> Self {
        self.quick_fixes = fixes;
        self
    }

    /// Adds a single quick fix.
    pub fn add_quick_fix(&mut self, fix: QuickFix) {
        self.quick_fixes.push(fix);
    }

    /// Attaches a documentation link placeholder.
    pub fn with_documentation_link(mut self, link: impl Into<String>) -> Self {
        self.documentation_link = Some(link.into());
        self
    }

    /// Explicitly sets custom unique ID.
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = id.into();
        self
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}] {} {} at {}:{}: {}",
            self.language,
            self.severity.symbol(),
            self.rule_name,
            self.line,
            self.column,
            self.message
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostics::position::DiagnosticPosition;

    #[test]
    fn test_diagnostic_object_construction() {
        let p1 = DiagnosticPosition::new(5, 12, 45);
        let p2 = DiagnosticPosition::new(5, 20, 53);
        let range = DiagnosticRange::new(p1, p2);

        let diag = Diagnostic::new(
            DiagnosticLevel::Security,
            "Exposed sensitive secret key",
            range,
            "SecNoHardcodedSecrets",
            LanguageId::Yaml,
        )
        .with_description("Secrets should never be checked into version control.")
        .with_documentation_link("https://gic.dev/docs/rules/sec-no-hardcoded-secrets");

        assert_eq!(diag.line, 5);
        assert_eq!(diag.column, 12);
        assert_eq!(diag.severity, DiagnosticLevel::Security);
        assert_eq!(diag.rule_name, "SecNoHardcodedSecrets");
        assert_eq!(diag.language, LanguageId::Yaml);
        assert!(diag.description.is_some());
        assert!(diag.documentation_link.is_some());
        assert!(diag.timestamp > 0);
    }
}
