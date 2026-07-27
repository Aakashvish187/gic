//! SELinux Configuration Analyzer.
//!
//! Validates `/etc/selinux/config` and detects disabled SELinux states.

use crate::linux::errors::LinuxResult;
use crate::yaml::parser::{Position, Span};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelinuxDiagnostic {
    pub rule_id: String,
    pub message: String,
    pub span: Span,
    pub is_error: bool,
}

#[derive(Debug, Clone, Default)]
pub struct SelinuxAnalyzer;

impl SelinuxAnalyzer {
    pub fn new() -> Self {
        Self
    }

    pub fn analyze_config(&self, source: &str) -> LinuxResult<Vec<SelinuxDiagnostic>> {
        let mut diagnostics = Vec::new();
        for (idx, line) in source.lines().enumerate() {
            let line_num = idx + 1;
            let trimmed = line.trim();
            if trimmed.starts_with("SELINUX=disabled") {
                let pos = Position::new(line_num, 1, 0);
                diagnostics.push(SelinuxDiagnostic {
                    rule_id: "sec-selinux-disabled".to_string(),
                    message:
                        "SELinux is disabled. It is recommended to use 'enforcing' or 'permissive'."
                            .to_string(),
                    span: Span::new(pos, pos),
                    is_error: false, // Warning
                });
            }
        }
        Ok(diagnostics)
    }
}
