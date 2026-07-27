//! YAML security diagnostic adapter.

use crate::diagnostics::{Diagnostic, DiagnosticLevel};
use crate::security::category::SecurityCategory;
use crate::security::evidence::FindingEvidence;
use crate::security::findings::SecurityFinding;
use crate::security::severity::SecuritySeverity;

/// Adapter ingesting YAML diagnostics from `YamlEngine`.
#[derive(Debug, Clone, Default)]
pub struct YamlSecurityAdapter;

impl YamlSecurityAdapter {
    pub fn new() -> Self {
        Self
    }

    /// Translates YAML diagnostics into normalized `SecurityFinding` entries.
    pub fn convert_diagnostics(&self, diagnostics: &[Diagnostic]) -> Vec<SecurityFinding> {
        let mut findings = Vec::new();

        for diag in diagnostics {
            if diag.rule_name.starts_with("YAML") || diag.rule_name.starts_with("YML") {
                let severity = match diag.severity {
                    DiagnosticLevel::Security | DiagnosticLevel::Error => SecuritySeverity::Medium,
                    DiagnosticLevel::Warning => SecuritySeverity::Low,
                    _ => SecuritySeverity::Information,
                };

                let evidence = FindingEvidence {
                    file_path: None,
                    range: diag.range,
                    snippet: diag.message.clone(),
                    rule_id: diag.rule_name.clone(),
                    source_engine: "YamlEngine".to_string(),
                };

                findings.push(SecurityFinding::new(
                    severity,
                    SecurityCategory::Configuration,
                    format!("YAML Security Misconfiguration: {}", diag.rule_name),
                    diag.message.clone(),
                    evidence,
                    "Review YAML configuration for insecure or unencrypted values.",
                ));
            }
        }

        findings
    }
}
