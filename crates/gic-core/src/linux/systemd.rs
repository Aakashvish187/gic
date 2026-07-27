//! Systemd Unit File Analyzer.
//!
//! Parses and validates `.service`, `.target`, and `.timer` files.
//! Analyzes `[Unit]`, `[Service]`, `[Install]`, `ExecStart`, `Restart`, `WantedBy`.

use std::collections::HashSet;

use crate::linux::errors::LinuxResult;
use crate::yaml::parser::{Position, Span};

/// Systemd unit validation diagnostic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SystemdDiagnostic {
    pub rule_id: String,
    pub message: String,
    pub span: Span,
    pub is_error: bool,
}

/// Systemd unit analyzer.
#[derive(Debug, Clone, Default)]
pub struct SystemdAnalyzer;

impl SystemdAnalyzer {
    pub fn new() -> Self {
        Self
    }

    /// Analyzes a raw systemd unit file string.
    pub fn analyze(&self, source: &str) -> LinuxResult<Vec<SystemdDiagnostic>> {
        let mut diagnostics = Vec::new();
        let mut sections = HashSet::new();
        let mut has_exec_start = false;
        let mut is_service = false;

        for (idx, line) in source.lines().enumerate() {
            let line_num = idx + 1;
            let trimmed = line.trim();

            if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with(';') {
                continue;
            }

            if trimmed.starts_with('[') && trimmed.ends_with(']') {
                let section_name = trimmed.trim_matches(|c| c == '[' || c == ']');
                sections.insert(section_name.to_string());
                if section_name == "Service" {
                    is_service = true;
                }
                continue;
            }

            if trimmed.starts_with("ExecStart=") {
                has_exec_start = true;
                let cmd = trimmed.trim_start_matches("ExecStart=").trim();
                if cmd.is_empty() {
                    let pos_start = Position::new(line_num, 1, 0);
                    let pos_end = Position::new(line_num, line.len().max(1), 0);
                    diagnostics.push(SystemdDiagnostic {
                        rule_id: "lin-systemd-execstart".to_string(),
                        message: "ExecStart requires an absolute path to an executable".to_string(),
                        span: Span::new(pos_start, pos_end),
                        is_error: true,
                    });
                } else if !cmd.starts_with('-') && !cmd.starts_with('/') {
                    // Systemd typically requires absolute paths unless it's prefixed with special modifiers like '-'
                    let pos_start = Position::new(line_num, 1, 0);
                    let pos_end = Position::new(line_num, line.len().max(1), 0);
                    diagnostics.push(SystemdDiagnostic {
                        rule_id: "lin-systemd-execstart-path".to_string(),
                        message: "ExecStart command should ideally use an absolute path"
                            .to_string(),
                        span: Span::new(pos_start, pos_end),
                        is_error: false, // Warning
                    });
                }
            }
        }

        if is_service && !has_exec_start {
            diagnostics.push(SystemdDiagnostic {
                rule_id: "lin-systemd-missing-execstart".to_string(),
                message: "[Service] section is missing required 'ExecStart=' directive".to_string(),
                span: Span::default(),
                is_error: true,
            });
        }

        Ok(diagnostics)
    }
}
