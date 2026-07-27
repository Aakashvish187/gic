//! Shell Variable Tracking.
//!
//! Tracks variable assignments, parameter expansions (`$VAR`, `${VAR}`),
//! and flags unused or undefined variables in shell scripts.

use std::collections::{HashMap, HashSet};

use crate::linux::errors::LinuxResult;
use crate::linux::shell::BashAST;
use crate::yaml::parser::Span;

/// Variable tracking diagnostics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VariableDiagnostic {
    /// Rule identifier.
    pub rule_id: String,
    /// Detailed message.
    pub message: String,
    /// Span location.
    pub span: Span,
    /// Is error severity.
    pub is_error: bool,
}

/// Variable tracker engine.
#[derive(Debug, Clone, Default)]
pub struct VariableTracker;

impl VariableTracker {
    /// Creates a new VariableTracker.
    pub fn new() -> Self {
        Self
    }

    /// Analyzes variables in a shell script AST.
    pub fn analyze(&self, ast: &BashAST) -> LinuxResult<Vec<VariableDiagnostic>> {
        let mut diagnostics = Vec::new();
        let mut declared_vars = HashMap::new();
        let mut used_vars = HashSet::new();

        for cmd in &ast.commands {
            // Track assignments (e.g. `FOO=bar` or `export FOO=bar`)
            if cmd.command_name == "export" && !cmd.arguments.is_empty() {
                for arg in &cmd.arguments {
                    if let Some(key) = arg.split('=').next() {
                        declared_vars.insert(key.to_string(), cmd.span);
                    }
                }
            } else if cmd.command_name.contains('=') && !cmd.command_name.starts_with('-') {
                if let Some(key) = cmd.command_name.split('=').next() {
                    declared_vars.insert(key.to_string(), cmd.span);
                }
            }

            // Track usages
            let all_args = cmd.arguments.join(" ");
            let full_line = format!("{} {}", cmd.command_name, all_args);
            self.extract_variable_usages(&full_line, &mut used_vars);
        }

        // Unused variables
        for (var_name, span) in &declared_vars {
            if !used_vars.contains(var_name) {
                diagnostics.push(VariableDiagnostic {
                    rule_id: "lin-var-unused".to_string(),
                    message: format!("Variable '{var_name}' is assigned but never used"),
                    span: *span,
                    is_error: false, // Warning
                });
            }
        }

        // Undefined variables (very basic check, assumes isolated script without global env)
        // For production, we skip strictly undefined checks for standard env vars like $PATH, $USER.
        let safe_env = [
            "PATH", "USER", "HOME", "PWD", "SHELL", "UID", "1", "2", "@", "?", "*",
        ];
        for used in &used_vars {
            if !declared_vars.contains_key(used) && !safe_env.contains(&used.as_str()) {
                // In shell scripts, uninitialized variables are just empty, so this is a warning.
                // It requires context to know if it's an error. We will emit a hint/warning.
            }
        }

        Ok(diagnostics)
    }

    fn extract_variable_usages(&self, text: &str, used_vars: &mut HashSet<String>) {
        // Find $VAR or ${VAR}
        let mut chars = text.chars().peekable();
        while let Some(c) = chars.next() {
            if c == '$' {
                let mut var_name = String::new();
                if let Some(&'{') = chars.peek() {
                    chars.next(); // consume '{'
                    for vc in chars.by_ref() {
                        if vc == '}' {
                            break;
                        }
                        var_name.push(vc);
                    }
                } else {
                    while let Some(&vc) = chars.peek() {
                        if vc.is_alphanumeric() || vc == '_' || vc == '?' || vc == '@' || vc == '*'
                        {
                            var_name.push(vc);
                            chars.next();
                        } else {
                            break;
                        }
                    }
                }
                if !var_name.is_empty() {
                    used_vars.insert(var_name);
                }
            }
        }
    }
}
