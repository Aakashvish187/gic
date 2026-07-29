//! Linux security diagnostic adapter.

use crate::diagnostics::{Diagnostic, DiagnosticLevel};
use crate::security::category::SecurityCategory;
use crate::security::evidence::FindingEvidence;
use crate::security::findings::SecurityFinding;
use crate::security::severity::SecuritySeverity;

/// Adapter ingesting Linux security diagnostics from `LinuxEngine`.
#[derive(Debug, Clone, Default)]
pub struct LinuxSecurityAdapter;

impl LinuxSecurityAdapter {
    pub fn new() -> Self {
        Self
    }

    /// Translates Linux diagnostics into normalized `SecurityFinding` entries.
    pub fn convert_diagnostics(&self, diagnostics: &[Diagnostic]) -> Vec<SecurityFinding> {
        let mut findings = Vec::new();

        for diag in diagnostics {
            if diag.rule_name.contains("Linux")
                || diag.rule_name.starts_with("LNX")
                || diag.rule_name.contains("chmod")
            {
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
                    source_engine: "LinuxEngine".to_string(),
                };

                findings.push(SecurityFinding::new(
                    severity,
                    SecurityCategory::Linux,
                    format!("Linux Security: {}", diag.rule_name),
                    diag.message.clone(),
                    evidence,
                    "Enforce Linux system hardening and least-privilege permissions.",
                ));
            }
        }

        findings
    }
}
