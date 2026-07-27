//! Linux Networking Configuration Analyzer.
//!
//! Validates `/etc/resolv.conf`, `/etc/hosts`.

use crate::linux::errors::LinuxResult;
use crate::yaml::parser::{Position, Span};

/// Networking validation diagnostic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetworkDiagnostic {
    pub rule_id: String,
    pub message: String,
    pub span: Span,
    pub is_error: bool,
}

/// Networking configuration analyzer.
#[derive(Debug, Clone, Default)]
pub struct NetworkAnalyzer;

impl NetworkAnalyzer {
    pub fn new() -> Self {
        Self
    }

    /// Analyzes `/etc/hosts`.
    pub fn analyze_hosts(&self, source: &str) -> LinuxResult<Vec<NetworkDiagnostic>> {
        let mut diagnostics = Vec::new();

        for (idx, line) in source.lines().enumerate() {
            let line_num = idx + 1;
            let trimmed = line.trim();

            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if parts.len() < 2 {
                let pos_start = Position::new(line_num, 1, 0);
                let pos_end = Position::new(line_num, line.len().max(1), 0);
                diagnostics.push(NetworkDiagnostic {
                    rule_id: "lin-net-hosts".to_string(),
                    message: "Invalid hosts entry: requires IP address and at least one hostname"
                        .to_string(),
                    span: Span::new(pos_start, pos_end),
                    is_error: true,
                });
            }
        }

        Ok(diagnostics)
    }
}
