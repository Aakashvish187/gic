//! Linux Environment Configuration Analyzer.
//!
//! Parses and validates `/etc/environment`, `~/.bashrc`, `/etc/profile`,
//! checking for syntax issues in key-value environment assignments.

use crate::linux::errors::LinuxResult;
use crate::yaml::parser::{Position, Span};

/// Environment file validation diagnostic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnvironmentDiagnostic {
    /// Rule identifier.
    pub rule_id: String,
    /// Detailed message.
    pub message: String,
    /// Span location.
    pub span: Span,
    /// Is error severity.
    pub is_error: bool,
}

/// Environment configuration analyzer.
#[derive(Debug, Clone, Default)]
pub struct EnvironmentAnalyzer;

impl EnvironmentAnalyzer {
    /// Creates a new EnvironmentAnalyzer.
    pub fn new() -> Self {
        Self
    }

    /// Analyzes a raw environment file string.
    pub fn analyze(&self, source: &str) -> LinuxResult<Vec<EnvironmentDiagnostic>> {
        let mut diagnostics = Vec::new();

        for (idx, line) in source.lines().enumerate() {
            let line_num = idx + 1;
            let trimmed = line.trim();

            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            // Valid lines must contain an '=' for environment files.
            // Export prefix is allowed in .bashrc/.profile but optional in /etc/environment.
            let no_export = trimmed.trim_start_matches("export ");
            if !no_export.contains('=') {
                let pos_start = Position::new(line_num, 1, 0);
                let pos_end = Position::new(line_num, line.len().max(1), 0);
                diagnostics.push(EnvironmentDiagnostic {
                    rule_id: "lin-env-syntax".to_string(),
                    message: "Invalid environment variable assignment syntax: missing '='".to_string(),
                    span: Span::new(pos_start, pos_end),
                    is_error: true,
                });
            } else {
                let parts: Vec<&str> = no_export.splitn(2, '=').collect();
                if let Some(key) = parts.first() {
                    let key = key.trim();
                    if key.is_empty() || key.contains(' ') || key.contains('-') {
                        let pos_start = Position::new(line_num, 1, 0);
                        let pos_end = Position::new(line_num, line.len().max(1), 0);
                        diagnostics.push(EnvironmentDiagnostic {
                            rule_id: "lin-env-key".to_string(),
                            message: format!("Invalid environment variable name: '{}'", key),
                            span: Span::new(pos_start, pos_end),
                            is_error: true,
                        });
                    }
                }
            }
        }

        Ok(diagnostics)
    }
}
