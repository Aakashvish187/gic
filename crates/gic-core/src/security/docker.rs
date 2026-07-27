//! Docker security diagnostic adapter.

use crate::diagnostics::{Diagnostic, DiagnosticLevel};
use crate::security::category::SecurityCategory;
use crate::security::evidence::FindingEvidence;
use crate::security::findings::SecurityFinding;
use crate::security::severity::SecuritySeverity;

/// Adapter ingesting Docker security diagnostics from `DockerEngine`.
#[derive(Debug, Clone, Default)]
pub struct DockerSecurityAdapter;

impl DockerSecurityAdapter {
    pub fn new() -> Self {
        Self
    }

    /// Translates Docker diagnostics into normalized `SecurityFinding` entries.
    pub fn convert_diagnostics(&self, diagnostics: &[Diagnostic]) -> Vec<SecurityFinding> {
        let mut findings = Vec::new();

        for diag in diagnostics {
            if diag.rule_name.contains("Docker") || diag.rule_name.starts_with("DOC") {
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
                    source_engine: "DockerEngine".to_string(),
                };

                findings.push(SecurityFinding::new(
                    severity,
                    SecurityCategory::Containers,
                    format!("Docker Security: {}", diag.rule_name),
                    diag.message.clone(),
                    evidence,
                    "Follow Docker container security hardening guidelines.",
                ));
            }
        }

        findings
    }
}
