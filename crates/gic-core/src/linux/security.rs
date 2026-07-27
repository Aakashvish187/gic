//! IaC & Linux Security Audit Engine.
//!
//! Evaluates Bash scripts and Linux config for dangerous behavior:
//! `rm -rf /`, `curl | bash`, etc.

use crate::linux::errors::LinuxResult;
use crate::linux::shell::BashAST;
use crate::yaml::parser::Span;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecurityDiagnostic {
    pub rule_id: String,
    pub message: String,
    pub span: Span,
    pub is_error: bool,
}

#[derive(Debug, Clone, Default)]
pub struct SecurityAnalyzer;

impl SecurityAnalyzer {
    pub fn new() -> Self {
        Self
    }

    pub fn analyze_bash(&self, ast: &BashAST) -> LinuxResult<Vec<SecurityDiagnostic>> {
        let mut diagnostics = Vec::new();

        for cmd in &ast.commands {
            // Check rm -rf /
            if cmd.command_name == "rm" {
                let has_rf = cmd.arguments.iter().any(|a| {
                    a == "-rf"
                        || a == "-fr"
                        || (a.contains('r') && a.contains('f') && a.starts_with('-'))
                });
                let has_root = cmd.arguments.iter().any(|a| a == "/" || a == "/*");
                if has_rf && has_root {
                    diagnostics.push(SecurityDiagnostic {
                        rule_id: "sec-bash-rm-rf-root".to_string(),
                        message: "Extremely dangerous command: recursively deleting root directory"
                            .to_string(),
                        span: cmd.span,
                        is_error: true,
                    });
                }
            }

            // Check curl | bash
            if (cmd.command_name == "curl" || cmd.command_name == "wget")
                && cmd.is_piped {
                    let piped_to_bash = cmd.pipeline_commands.iter().any(|pipe_cmd| {
                        let trimmed = pipe_cmd.trim();
                        trimmed.starts_with("bash") || trimmed.starts_with("sh")
                    });
                    if piped_to_bash {
                        diagnostics.push(SecurityDiagnostic {
                            rule_id: "sec-bash-curl-pipe-bash".to_string(),
                            message: "Dangerous execution: downloading and immediately executing a script via pipe".to_string(),
                            span: cmd.span,
                            is_error: true,
                        });
                    }
                }
        }

        Ok(diagnostics)
    }
}
