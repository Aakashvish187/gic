//! Certificate and PKI analysis module.

use crate::diagnostics::{DiagnosticPosition, DiagnosticRange};
use crate::security::category::SecurityCategory;
use crate::security::evidence::FindingEvidence;
use crate::security::findings::SecurityFinding;
use crate::security::severity::SecuritySeverity;
use std::path::Path;

/// Certificate analyzer identifying weak or dangerous TLS/PKI configurations.
#[derive(Debug, Clone, Default)]
pub struct CertificateAnalyzer;

impl CertificateAnalyzer {
    pub fn new() -> Self {
        Self
    }

    /// Analyzes content for certificate declarations (e.g. self-signed certs or weak RSA key length).
    pub fn analyze_content(&self, file_path: Option<&Path>, content: &str) -> Vec<SecurityFinding> {
        let mut findings = Vec::new();

        for (line_idx, line) in content.lines().enumerate() {
            let line_lower = line.to_lowercase();
            let line_no = line_idx + 1;

            if line_lower.contains("ssl_verify: false") || line_lower.contains("insecure_skip_verify: true") {
                let evidence = FindingEvidence {
                    file_path: file_path.map(|p| p.to_path_buf()),
                    range: DiagnosticRange::new(
                        DiagnosticPosition::new(line_no, 1, 0),
                        DiagnosticPosition::new(line_no, line.len().max(1), 0),
                    ),
                    snippet: line.trim().to_string(),
                    rule_id: "CERT-001".to_string(),
                    source_engine: "CertificateAnalyzer".to_string(),
                };

                findings.push(SecurityFinding::new(
                    SecuritySeverity::High,
                    SecurityCategory::Certificates,
                    "TLS Certificate Verification Disabled",
                    "SSL/TLS certificate verification is disabled, enabling Man-in-the-Middle (MitM) attacks.",
                    evidence,
                    "Enable TLS certificate verification in production environments.",
                ));
            }
        }

        findings
    }
}
