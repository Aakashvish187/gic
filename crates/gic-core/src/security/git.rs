//! Git security diagnostic adapter.

use crate::diagnostics::{Diagnostic, DiagnosticLevel};
use crate::security::category::SecurityCategory;
use crate::security::evidence::FindingEvidence;
use crate::security::findings::SecurityFinding;
use crate::security::severity::SecuritySeverity;

/// Adapter ingesting Git diagnostics from `GitEngine`.
#[derive(Debug, Clone, Default)]
pub struct GitSecurityAdapter;

impl GitSecurityAdapter {
    pub fn new() -> Self {
        Self
    }

    /// Translates Git diagnostics into normalized `SecurityFinding` entries.
    pub fn convert_diagnostics(&self, diagnostics: &[Diagnostic]) -> Vec<SecurityFinding> {
        let mut findings = Vec::new();

        for diag in diagnostics {
            if diag.rule_name.starts_with("GIT") {
                let severity = match diag.severity {
                    DiagnosticLevel::Security | DiagnosticLevel::Error => SecuritySeverity::High,
                    DiagnosticLevel::Warning => SecuritySeverity::Medium,
                    _ => SecuritySeverity::Low,
                };

                let evidence = FindingEvidence {
                    file_path: None,
                    range: diag.range,
                    snippet: diag.message.clone(),
                    rule_id: diag.rule_name.clone(),
                    source_engine: "GitEngine".to_string(),
                };

                findings.push(SecurityFinding::new(
                    severity,
                    SecurityCategory::AccessControl,
                    format!("Git Repository Risk: {}", diag.rule_name),
                    diag.message.clone(),
                    evidence,
                    "Resolve uncommitted changes or detached HEAD states before deployment.",
                ));
            }
        }

        findings
    }
}
