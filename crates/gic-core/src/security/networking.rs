//! Networking security diagnostic adapter.

use crate::diagnostics::{Diagnostic, DiagnosticLevel};
use crate::security::category::SecurityCategory;
use crate::security::evidence::FindingEvidence;
use crate::security::findings::SecurityFinding;
use crate::security::severity::SecuritySeverity;

/// Adapter ingesting Networking diagnostics from `NetworkEngine`.
#[derive(Debug, Clone, Default)]
pub struct NetworkSecurityAdapter;

impl NetworkSecurityAdapter {
    pub fn new() -> Self {
        Self
    }

    /// Translates networking diagnostics into normalized `SecurityFinding` entries.
    pub fn convert_diagnostics(&self, diagnostics: &[Diagnostic]) -> Vec<SecurityFinding> {
        let mut findings = Vec::new();

        for diag in diagnostics {
            if diag.rule_name.starts_with("NET") || diag.rule_name.contains("Port") || diag.rule_name.contains("CIDR") {
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
                    source_engine: "NetworkEngine".to_string(),
                };

                findings.push(SecurityFinding::new(
                    severity,
                    SecurityCategory::Networking,
                    format!("Network Security: {}", diag.rule_name),
                    diag.message.clone(),
                    evidence,
                    "Restrict network exposure and apply least-privilege firewall rules.",
                ));
            }
        }

        findings
    }
}
