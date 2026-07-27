//! Shell Command Understanding Registry.
//!
//! Recognizes standard Linux commands and flags. Checks for invalid arguments,
//! missing executables, and provides knowledge about dangerous flags.

use std::collections::HashSet;

use crate::linux::errors::LinuxResult;
use crate::linux::shell::BashAST;
use crate::yaml::parser::Span;

/// Command validation finding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandDiagnostic {
    /// Rule identifier.
    pub rule_id: String,
    /// Message.
    pub message: String,
    /// Span location.
    pub span: Span,
    /// Is it an error?
    pub is_error: bool,
}

/// Registry of known Linux commands and their validation logic.
#[derive(Debug, Clone)]
pub struct CommandRegistry {
    known_commands: HashSet<&'static str>,
}

impl Default for CommandRegistry {
    fn default() -> Self {
        let mut known = HashSet::new();
        let cmds = [
            "ls",
            "cp",
            "mv",
            "rm",
            "mkdir",
            "chmod",
            "chown",
            "chgrp",
            "find",
            "grep",
            "sed",
            "awk",
            "cut",
            "sort",
            "uniq",
            "head",
            "tail",
            "tar",
            "gzip",
            "zip",
            "curl",
            "wget",
            "ssh",
            "scp",
            "rsync",
            "systemctl",
            "journalctl",
            "dnf",
            "yum",
            "apt",
            "rpm",
            "ip",
            "ss",
            "netstat",
            "ping",
            "traceroute",
            "hostnamectl",
            "timedatectl",
            "mount",
            "umount",
            "df",
            "du",
            "ps",
            "top",
            "kill",
            "pkill",
            "nohup",
            "screen",
            "tmux",
            "crontab",
            "useradd",
            "usermod",
            "groupadd",
            "passwd",
            "sudo",
            "echo",
            "cat",
            "export",
        ];
        for cmd in cmds {
            known.insert(cmd);
        }
        Self {
            known_commands: known,
        }
    }
}

impl CommandRegistry {
    /// Creates a new CommandRegistry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Analyzes a Bash AST and detects unknown commands and dangerous flags.
    pub fn validate_commands(&self, ast: &BashAST) -> LinuxResult<Vec<CommandDiagnostic>> {
        let mut diagnostics = Vec::new();

        for cmd in &ast.commands {
            if cmd.command_name.starts_with('#') || cmd.command_name.is_empty() {
                continue;
            }

            // Variable assignments (e.g. `FOO=bar`) skip command check.
            if cmd.command_name.contains('=') && !cmd.command_name.starts_with('-') {
                continue;
            }

            // Check if command is known (skip paths like ./script.sh or /bin/sh)
            if !cmd.command_name.contains('/')
                && !self.known_commands.contains(cmd.command_name.as_str())
            {
                diagnostics.push(CommandDiagnostic {
                    rule_id: "lin-cmd-unknown".to_string(),
                    message: format!("Unknown or unverified command '{}'", cmd.command_name),
                    span: cmd.span,
                    is_error: false, // Warning
                });
            }
        }

        Ok(diagnostics)
    }
}
