//! Kubernetes security diagnostic adapter.

use crate::diagnostics::{Diagnostic, DiagnosticLevel};
use crate::security::category::SecurityCategory;
use crate::security::evidence::FindingEvidence;
use crate::security::findings::SecurityFinding;
use crate::security::severity::SecuritySeverity;

/// Adapter ingesting Kubernetes security diagnostics from `K8sEngine`.
#[derive(Debug, Clone, Default)]
pub struct K8sSecurityAdapter;

impl K8sSecurityAdapter {
    pub fn new() -> Self {
        Self
    }

    /// Translates K8s diagnostics into normalized `SecurityFinding` entries.
    pub fn convert_diagnostics(&self, diagnostics: &[Diagnostic]) -> Vec<SecurityFinding> {
        let mut findings = Vec::new();

        for diag in diagnostics {
            if diag.rule_name.contains("K8s") || diag.rule_name.starts_with("K8S") {
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
                    source_engine: "K8sEngine".to_string(),
                };

                findings.push(SecurityFinding::new(
                    severity,
                    SecurityCategory::Kubernetes,
                    format!("Kubernetes Security: {}", diag.rule_name),
                    diag.message.clone(),
                    evidence,
                    "Enforce Kubernetes Pod Security Standards and SecurityContext settings.",
                ));
            }
        }

        findings
    }
}
