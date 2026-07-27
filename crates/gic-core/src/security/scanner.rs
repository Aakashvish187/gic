//! Unified security scanner orchestrating all individual scan passes.

use crate::security::certificates::CertificateAnalyzer;
use crate::security::credentials::CredentialsAnalyzer;
use crate::security::errors::SecurityResult;
use crate::security::findings::SecurityFinding;
use crate::security::secrets::SecretScanner;
use std::path::Path;

/// Unified scanner running all sub-scanners across a text buffer.
#[derive(Debug, Clone)]
pub struct SecurityScanner {
    secret_scanner: SecretScanner,
    credentials_analyzer: CredentialsAnalyzer,
    certificate_analyzer: CertificateAnalyzer,
}

impl SecurityScanner {
    /// Constructs a `SecurityScanner` with all detection sub-systems.
    pub fn new() -> SecurityResult<Self> {
        Ok(Self {
            secret_scanner: SecretScanner::new()?,
            credentials_analyzer: CredentialsAnalyzer::new(),
            certificate_analyzer: CertificateAnalyzer::new(),
        })
    }

    /// Scans a file content buffer and returns all detected security findings.
    pub fn scan_content(&self, file_path: Option<&Path>, content: &str) -> Vec<SecurityFinding> {
        let mut findings = Vec::new();

        // Skip large binary-like content (> 1MB)
        if content.len() > 1_048_576 {
            return findings;
        }

        findings.extend(self.secret_scanner.scan_buffer(file_path, content));
        findings.extend(self.credentials_analyzer.analyze_content(file_path, content));
        findings.extend(self.certificate_analyzer.analyze_content(file_path, content));

        findings
    }
}

impl Default for SecurityScanner {
    fn default() -> Self {
        Self::new().expect("SecurityScanner initialization failed")
    }
}
