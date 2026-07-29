//! Security diagnostics adapter — maps `SecurityFinding` into GIC `Diagnostic` objects
//! for editor UI integration (Problems Panel, Hover, Status Bar).

use crate::diagnostics::{Diagnostic, DiagnosticLevel};
use crate::parser::LanguageId;
use crate::security::findings::SecurityFinding;
use crate::security::severity::SecuritySeverity;

/// Adapter converting `SecurityFinding` into GIC `Diagnostic` objects.
#[derive(Debug, Clone, Default)]
pub struct SecurityDiagnostics;

impl SecurityDiagnostics {
    pub fn new() -> Self {
        Self
    }

    /// Converts all security findings into GIC `Diagnostic` objects for the Problems Panel.
    pub fn to_diagnostics(&self, findings: &[SecurityFinding]) -> Vec<Diagnostic> {
        findings
            .iter()
            .map(|f| self.finding_to_diagnostic(f))
            .collect()
    }

    fn finding_to_diagnostic(&self, finding: &SecurityFinding) -> Diagnostic {
        let level = match finding.severity {
            SecuritySeverity::Critical | SecuritySeverity::High => DiagnosticLevel::Security,
            SecuritySeverity::Medium => DiagnosticLevel::Warning,
            SecuritySeverity::Low => DiagnosticLevel::Hint,
            SecuritySeverity::Information => DiagnosticLevel::Information,
        };

        Diagnostic::new(
            level,
            format!("{}: {}", finding.category, finding.description),
            finding.evidence.range,
            finding.evidence.rule_id.clone(),
            LanguageId::PlainText,
        )
        .with_description(finding.remediation.clone())
    }
}
