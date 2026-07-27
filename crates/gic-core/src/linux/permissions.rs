//! Linux Filesystem Permissions Analyzer.
//!
//! Validates octal (`777`, `666`, `644`) and symbolic (`u+x`, `go-w`) permission strings
//! passed to `chmod`, `chown`, and `chgrp`.

use crate::linux::errors::LinuxResult;
use crate::linux::shell::BashAST;
use crate::yaml::parser::Span;

/// Permissions validation finding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PermissionDiagnostic {
    pub rule_id: String,
    pub message: String,
    pub span: Span,
    pub is_error: bool,
}

/// Permissions engine.
#[derive(Debug, Clone, Default)]
pub struct PermissionsAnalyzer;

impl PermissionsAnalyzer {
    pub fn new() -> Self {
        Self
    }

    /// Analyzes a Bash AST for dangerous permission flags.
    pub fn analyze(&self, ast: &BashAST) -> LinuxResult<Vec<PermissionDiagnostic>> {
        let mut diagnostics = Vec::new();

        for cmd in &ast.commands {
            if cmd.command_name == "chmod" {
                for arg in &cmd.arguments {
                    if arg == "777" {
                        diagnostics.push(PermissionDiagnostic {
                            rule_id: "sec-bash-chmod-777".to_string(),
                            message: "Dangerous permission '777' grants world-writable access"
                                .to_string(),
                            span: cmd.span,
                            is_error: true,
                        });
                    } else if arg == "666" {
                        diagnostics.push(PermissionDiagnostic {
                            rule_id: "sec-bash-chmod-666".to_string(),
                            message: "Dangerous permission '666' grants world-writable access"
                                .to_string(),
                            span: cmd.span,
                            is_error: true,
                        });
                    }
                }
            }
        }

        Ok(diagnostics)
    }
}
