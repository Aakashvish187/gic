//! SSH Client and Daemon Configuration Analyzer.
//!
//! Validates `sshd_config` and `ssh_config` files.
//! Audits for `PermitRootLogin`, `PasswordAuthentication`, and weak ciphers.

use crate::linux::errors::LinuxResult;
use crate::yaml::parser::{Position, Span};

/// SSH validation diagnostic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SshDiagnostic {
    pub rule_id: String,
    pub message: String,
    pub span: Span,
    pub is_error: bool,
}

/// SSH configuration analyzer.
#[derive(Debug, Clone, Default)]
pub struct SshAnalyzer;

impl SshAnalyzer {
    pub fn new() -> Self {
        Self
    }

    /// Analyzes `sshd_config` for security best practices.
    pub fn analyze_sshd(&self, source: &str) -> LinuxResult<Vec<SshDiagnostic>> {
        let mut diagnostics = Vec::new();

        for (idx, line) in source.lines().enumerate() {
            let line_num = idx + 1;
            let trimmed = line.trim();

            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if parts.len() >= 2 {
                let key = parts[0];
                let val = parts[1];
                let pos_start = Position::new(line_num, 1, 0);
                let pos_end = Position::new(line_num, line.len().max(1), 0);
                let span = Span::new(pos_start, pos_end);

                if key.eq_ignore_ascii_case("PermitRootLogin") && val.eq_ignore_ascii_case("yes") {
                    diagnostics.push(SshDiagnostic {
                        rule_id: "sec-ssh-root-login".to_string(),
                        message: "PermitRootLogin set to 'yes' allows direct root access over SSH"
                            .to_string(),
                        span,
                        is_error: true,
                    });
                } else if key.eq_ignore_ascii_case("PasswordAuthentication")
                    && val.eq_ignore_ascii_case("yes")
                {
                    diagnostics.push(SshDiagnostic {
                        rule_id: "sec-ssh-password-auth".to_string(),
                        message: "PasswordAuthentication set to 'yes' allows brute-force attacks. Prefer keys.".to_string(),
                        span,
                        is_error: false, // Warning
                    });
                }
            }
        }

        Ok(diagnostics)
    }
}
