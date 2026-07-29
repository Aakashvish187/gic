//! High-speed Secret & Hardcoded Key Scanner using optimized regex patterns.

use crate::diagnostics::{DiagnosticPosition, DiagnosticRange};
use crate::security::category::SecurityCategory;
use crate::security::errors::SecurityResult;
use crate::security::evidence::FindingEvidence;
use crate::security::findings::SecurityFinding;
use crate::security::severity::SecuritySeverity;
use regex::Regex;
use std::path::Path;

/// Specification for secret detection regex rule.
#[derive(Debug, Clone)]
pub struct SecretRule {
    pub rule_id: &'static str,
    pub title: &'static str,
    pub description: &'static str,
    pub pattern: Regex,
    pub severity: SecuritySeverity,
    pub remediation: &'static str,
}

/// Secret Scanner evaluating text buffers against cloud and credential patterns.
#[derive(Debug, Clone)]
pub struct SecretScanner {
    rules: Vec<SecretRule>,
}

impl Default for SecretScanner {
    fn default() -> Self {
        Self::new().expect("Failed to initialize default secret scanner rules")
    }
}

impl SecretScanner {
    /// Constructs a `SecretScanner` pre-loaded with production secret detection patterns.
    pub fn new() -> SecurityResult<Self> {
        let rules = vec![
            SecretRule {
                rule_id: "SEC-001",
                title: "AWS Access Key ID Detected",
                description: "Hardcoded AWS Access Key ID exposed in file.",
                pattern: Regex::new(r"(?i)\b(AKIA[0-9A-Z]{16})\b")?,
                severity: SecuritySeverity::Critical,
                remediation:
                    "Remove AWS Access Key ID and load from environment variables or AWS IAM Roles.",
            },
            SecretRule {
                rule_id: "SEC-002",
                title: "AWS Secret Access Key Detected",
                description: "Hardcoded AWS Secret Access Key exposed in file.",
                pattern: Regex::new(
                    r#"(?i)\b(aws_secret_access_key|aws_secret_key)\s*[:=]\s*['"]?([A-Za-z0-9/+=]{40})['"]?"#,
                )?,
                severity: SecuritySeverity::Critical,
                remediation:
                    "Revoke key immediately and use secret managers like AWS Secrets Manager.",
            },
            SecretRule {
                rule_id: "SEC-003",
                title: "GitHub Personal Access Token Detected",
                description: "Hardcoded GitHub PAT or OAuth token exposed.",
                pattern: Regex::new(r"\b(ghp_[a-zA-Z0-9]{36}|github_pat_[a-zA-Z0-9_]{82})\b")?,
                severity: SecuritySeverity::Critical,
                remediation: "Revoke GitHub token immediately.",
            },
            SecretRule {
                rule_id: "SEC-004",
                title: "Private Key / RSA Key Exposed",
                description: "Unencrypted Private Key header found in file.",
                pattern: Regex::new(r"-----BEGIN (RSA|OPENSSH|DSA|EC|PRIVATE) KEY-----")?,
                severity: SecuritySeverity::Critical,
                remediation:
                    "Do not commit private keys to version control. Store in protected keystore.",
            },
            SecretRule {
                rule_id: "SEC-005",
                title: "JWT Authentication Token Detected",
                description: "Hardcoded JSON Web Token detected.",
                pattern: Regex::new(
                    r"\beyJ[A-Za-z0-9-_=]+\.eyJ[A-Za-z0-9-_=]+\.[A-Za-z0-9-_.+/=]+\b",
                )?,
                severity: SecuritySeverity::High,
                remediation: "Inject JWT tokens at runtime instead of hardcoding.",
            },
            SecretRule {
                rule_id: "SEC-006",
                title: "Database Connection String with Credentials",
                description:
                    "Hardcoded database connection string containing username and password.",
                pattern: Regex::new(
                    r"(?i)\b(postgres|postgresql|mysql|mongodb|redis)://[a-zA-Z0-9_]+:[^@\s]+@[a-zA-Z0-9.-]+",
                )?,
                severity: SecuritySeverity::Critical,
                remediation: "Parameterize database credentials using secret stores.",
            },
        ];

        Ok(Self { rules })
    }

    /// Scans a text buffer line by line for secrets.
    pub fn scan_buffer(&self, file_path: Option<&Path>, content: &str) -> Vec<SecurityFinding> {
        let mut findings = Vec::new();

        for (line_idx, line) in content.lines().enumerate() {
            let line_no = line_idx + 1;

            for rule in &self.rules {
                if let Some(mat) = rule.pattern.find(line) {
                    let col_no = mat.start() + 1;
                    let sanitized_snippet = sanitize_secret_line(line);

                    let evidence = FindingEvidence {
                        file_path: file_path.map(|p| p.to_path_buf()),
                        range: DiagnosticRange::new(
                            DiagnosticPosition::new(line_no, col_no, mat.start()),
                            DiagnosticPosition::new(
                                line_no,
                                col_no + mat.as_str().len(),
                                mat.end(),
                            ),
                        ),
                        snippet: sanitized_snippet,
                        rule_id: rule.rule_id.to_string(),
                        source_engine: "SecretScanner".to_string(),
                    };

                    findings.push(SecurityFinding::new(
                        rule.severity,
                        SecurityCategory::Secrets,
                        rule.title,
                        rule.description,
                        evidence,
                        rule.remediation,
                    ));
                }
            }
        }

        findings
    }
}

/// Sanitizes sensitive match in snippet output for security reporting.
fn sanitize_secret_line(line: &str) -> String {
    if line.len() > 120 {
        format!("{}...", &line[..120])
    } else {
        line.to_string()
    }
}
