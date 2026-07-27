//! Database and API credentials analysis module.

use crate::diagnostics::{DiagnosticPosition, DiagnosticRange};
use crate::security::category::SecurityCategory;
use crate::security::evidence::FindingEvidence;
use crate::security::findings::SecurityFinding;
use crate::security::severity::SecuritySeverity;
use std::path::Path;

/// Credentials analyzer identifying hardcoded passwords and tokens.
#[derive(Debug, Clone, Default)]
pub struct CredentialsAnalyzer;

impl CredentialsAnalyzer {
    pub fn new() -> Self {
        Self
    }

    /// Analyzes text lines for password / API key assignments.
    pub fn analyze_content(&self, file_path: Option<&Path>, content: &str) -> Vec<SecurityFinding> {
        let mut findings = Vec::new();

        for (line_idx, line) in content.lines().enumerate() {
            let line_lower = line.to_lowercase();
            let line_no = line_idx + 1;

            if (line_lower.contains("password=") || line_lower.contains("password:") || line_lower.contains("secret_key="))
                && !line_lower.contains("${")
                && !line_lower.contains("env(")
                && !line_lower.contains("secretkeyref")
            {
                let evidence = FindingEvidence {
                    file_path: file_path.map(|p| p.to_path_buf()),
                    range: DiagnosticRange::new(
                        DiagnosticPosition::new(line_no, 1, 0),
                        DiagnosticPosition::new(line_no, line.len().max(1), 0),
                    ),
                    snippet: line.trim().to_string(),
                    rule_id: "CRED-001".to_string(),
                    source_engine: "CredentialsAnalyzer".to_string(),
                };

                findings.push(SecurityFinding::new(
                    SecuritySeverity::High,
                    SecurityCategory::Credentials,
                    "Hardcoded Password Assignment",
                    "A plain-text password assignment was detected in configuration source.",
                    evidence,
                    "Use secret management references (e.g. env vars, K8s secrets) instead of inline passwords.",
                ));
            }
        }

        findings
    }
}
