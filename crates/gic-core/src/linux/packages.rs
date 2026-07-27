//! Package Manager Command Analyzer.
//!
//! Validates `dnf`, `yum`, `apt`, `rpm` commands in shell scripts.

use crate::linux::errors::LinuxResult;
use crate::linux::shell::BashAST;
use crate::yaml::parser::Span;

/// Package management diagnostic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageDiagnostic {
    pub rule_id: String,
    pub message: String,
    pub span: Span,
    pub is_error: bool,
}

/// Package manager engine.
#[derive(Debug, Clone, Default)]
pub struct PackageAnalyzer;

impl PackageAnalyzer {
    pub fn new() -> Self {
        Self
    }

    /// Analyzes package manager calls in a Bash AST.
    pub fn analyze(&self, ast: &BashAST) -> LinuxResult<Vec<PackageDiagnostic>> {
        let mut diagnostics = Vec::new();

        for cmd in &ast.commands {
            if matches!(cmd.command_name.as_str(), "apt" | "apt-get" | "dnf" | "yum") {
                let has_yes = cmd
                    .arguments
                    .iter()
                    .any(|a| a == "-y" || a == "--yes" || a == "--assumeyes");
                if cmd.arguments.contains(&"install".to_string()) && !has_yes {
                    diagnostics.push(PackageDiagnostic {
                        rule_id: "lin-pkg-interactive".to_string(),
                        message: format!("Package manager '{} install' in script is missing '-y' (interactive prompt will block)", cmd.command_name),
                        span: cmd.span,
                        is_error: false, // Warning
                    });
                }
            }
        }

        Ok(diagnostics)
    }
}
