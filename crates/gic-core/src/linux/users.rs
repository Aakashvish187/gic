//! Linux User Configuration Parser.
//!
//! Parses and validates `/etc/passwd`, `/etc/shadow`, and `sudoers` rules.
//! Analyzes shell scripts for `useradd`, `usermod`, `passwd`, and `sudo` misuse.

use crate::linux::errors::LinuxResult;
use crate::yaml::parser::{Position, Span};

/// User validation diagnostic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserDiagnostic {
    pub rule_id: String,
    pub message: String,
    pub span: Span,
    pub is_error: bool,
}

/// User configuration analyzer.
#[derive(Debug, Clone, Default)]
pub struct UsersAnalyzer;

impl UsersAnalyzer {
    pub fn new() -> Self {
        Self
    }

    /// Analyzes `/etc/passwd` file syntax.
    pub fn analyze_passwd(&self, source: &str) -> LinuxResult<Vec<UserDiagnostic>> {
        let mut diagnostics = Vec::new();

        for (idx, line) in source.lines().enumerate() {
            let line_num = idx + 1;
            let trimmed = line.trim();

            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = trimmed.split(':').collect();
            if parts.len() != 7 {
                let pos_start = Position::new(line_num, 1, 0);
                let pos_end = Position::new(line_num, line.len().max(1), 0);
                diagnostics.push(UserDiagnostic {
                    rule_id: "lin-user-passwd".to_string(),
                    message: format!("Invalid /etc/passwd entry: requires exactly 7 fields separated by ':', found {}", parts.len()),
                    span: Span::new(pos_start, pos_end),
                    is_error: true,
                });
            }
        }

        Ok(diagnostics)
    }

    /// Analyzes `sudoers` file syntax.
    pub fn analyze_sudoers(&self, source: &str) -> LinuxResult<Vec<UserDiagnostic>> {
        let mut diagnostics = Vec::new();

        for (idx, line) in source.lines().enumerate() {
            let line_num = idx + 1;
            let trimmed = line.trim();

            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            if trimmed.contains("NOPASSWD: ALL") {
                let pos_start = Position::new(line_num, 1, 0);
                let pos_end = Position::new(line_num, line.len().max(1), 0);
                diagnostics.push(UserDiagnostic {
                    rule_id: "sec-sudoers-nopasswd-all".to_string(),
                    message: "Dangerous sudoers entry: NOPASSWD: ALL grants unrestricted root access without password".to_string(),
                    span: Span::new(pos_start, pos_end),
                    is_error: true, // Critical security issue
                });
            }
        }

        Ok(diagnostics)
    }
}
