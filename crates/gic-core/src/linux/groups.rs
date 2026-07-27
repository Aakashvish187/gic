//! Linux Group Configuration Parser.
//!
//! Parses and validates `/etc/group` and `groupadd` commands.

use crate::linux::errors::LinuxResult;
use crate::yaml::parser::{Position, Span};

/// Group validation diagnostic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GroupDiagnostic {
    pub rule_id: String,
    pub message: String,
    pub span: Span,
    pub is_error: bool,
}

/// Group configuration analyzer.
#[derive(Debug, Clone, Default)]
pub struct GroupsAnalyzer;

impl GroupsAnalyzer {
    pub fn new() -> Self {
        Self
    }

    /// Analyzes `/etc/group` file syntax.
    pub fn analyze_group(&self, source: &str) -> LinuxResult<Vec<GroupDiagnostic>> {
        let mut diagnostics = Vec::new();

        for (idx, line) in source.lines().enumerate() {
            let line_num = idx + 1;
            let trimmed = line.trim();

            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = trimmed.split(':').collect();
            if parts.len() != 4 {
                let pos_start = Position::new(line_num, 1, 0);
                let pos_end = Position::new(line_num, line.len().max(1), 0);
                diagnostics.push(GroupDiagnostic {
                    rule_id: "lin-group-syntax".to_string(),
                    message: format!("Invalid /etc/group entry: requires exactly 4 fields separated by ':', found {}", parts.len()),
                    span: Span::new(pos_start, pos_end),
                    is_error: true,
                });
            }
        }

        Ok(diagnostics)
    }
}
