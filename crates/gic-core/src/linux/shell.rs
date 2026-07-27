//! Bash & POSIX Shell Script Parser and AST Data Models.
//!
//! Scans raw shell script text, extracts shebang (`#!/bin/bash`, `#!/bin/sh`),
//! parses logical lines, pipeline commands, redirections (`>`, `>>`, `2>&1`, `|`), and subshells.

use crate::linux::errors::LinuxResult;
use crate::yaml::parser::{Position, Span};

/// Supported shell dialect kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ShellKind {
    /// GNU Bash (`#!/bin/bash` or `#!/usr/bin/env bash`).
    #[default]
    Bash,
    /// POSIX Shell (`#!/bin/sh`).
    PosixSh,
    /// Zsh (`#!/bin/zsh`).
    Zsh,
    /// Fish shell (`#!/usr/bin/fish`).
    Fish,
}

/// Shebang interpreter metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Shebang {
    /// Raw shebang line (e.g. `"#!/bin/bash"`).
    pub raw: String,
    /// Detected shell dialect.
    pub shell: ShellKind,
    /// Target span location.
    pub span: Span,
}

/// Parsed command invocation line within a shell script.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandInvocation {
    /// Executable or builtin command name (e.g. `"chmod"`, `"systemctl"`).
    pub command_name: String,
    /// Argument list passed to the command.
    pub arguments: Vec<String>,
    /// Raw unparsed line string.
    pub raw_line: String,
    /// 1-indexed line number in source file.
    pub line_number: usize,
    /// Pipeline components if command contains `|`.
    pub pipeline_commands: Vec<String>,
    /// Redirection operators present (`>`, `>>`, `<`).
    pub has_redirection: bool,
    /// Pipeline operator present (`|`).
    pub is_piped: bool,
    /// Span location.
    pub span: Span,
}

/// Complete parsed AST representation of a shell script.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BashAST {
    /// Optional shebang header.
    pub shebang: Option<Shebang>,
    /// Ordered command invocations.
    pub commands: Vec<CommandInvocation>,
    /// Original raw source code string.
    pub source: String,
}

impl BashAST {
    /// Returns true if the AST contains no command invocations.
    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }
}

/// High-performance zero-panic Bash parser.
#[derive(Debug, Clone, Default)]
pub struct BashParser;

impl BashParser {
    /// Creates a new BashParser.
    pub fn new() -> Self {
        Self
    }

    /// Parses raw shell script text into a `BashAST`.
    pub fn parse(&self, source: &str) -> LinuxResult<BashAST> {
        let mut shebang = None;
        let mut commands = Vec::new();
        let lines: Vec<&str> = source.lines().collect();

        for (idx, line) in lines.iter().enumerate() {
            let line_num = idx + 1;
            let trimmed = line.trim();

            if trimmed.is_empty() {
                continue;
            }

            if line_num == 1 && trimmed.starts_with("#!") {
                let shell = if trimmed.contains("sh") && !trimmed.contains("bash") {
                    ShellKind::PosixSh
                } else if trimmed.contains("zsh") {
                    ShellKind::Zsh
                } else if trimmed.contains("fish") {
                    ShellKind::Fish
                } else {
                    ShellKind::Bash
                };
                let pos_start = Position::new(1, 1, 0);
                let pos_end = Position::new(1, line.len().max(1), 0);
                shebang = Some(Shebang {
                    raw: trimmed.to_string(),
                    shell,
                    span: Span::new(pos_start, pos_end),
                });
                continue;
            }

            if trimmed.starts_with('#') {
                continue;
            }

            let is_piped = trimmed.contains('|');
            let has_redirection = trimmed.contains('>') || trimmed.contains('<');

            let pipeline_commands: Vec<String> = if is_piped {
                trimmed.split('|').map(|s| s.trim().to_string()).collect()
            } else {
                vec![trimmed.to_string()]
            };

            let first_cmd = pipeline_commands
                .first()
                .map(|s| s.as_str())
                .unwrap_or(trimmed);
            let tokens = tokenize_command_line(first_cmd);

            if !tokens.is_empty() {
                let command_name = tokens[0].clone();
                let arguments = tokens[1..].to_vec();

                let pos_start = Position::new(line_num, 1, 0);
                let pos_end = Position::new(line_num, line.len().max(1), 0);

                commands.push(CommandInvocation {
                    command_name,
                    arguments,
                    raw_line: line.to_string(),
                    line_number: line_num,
                    pipeline_commands,
                    has_redirection,
                    is_piped,
                    span: Span::new(pos_start, pos_end),
                });
            }
        }

        Ok(BashAST {
            shebang,
            commands,
            source: source.to_string(),
        })
    }
}

fn tokenize_command_line(line: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut in_quotes = false;
    let mut quote_char = ' ';
    let mut current = String::new();

    for c in line.chars() {
        match c {
            '"' | '\'' => {
                if in_quotes && c == quote_char {
                    in_quotes = false;
                } else if !in_quotes {
                    in_quotes = true;
                    quote_char = c;
                } else {
                    current.push(c);
                }
            }
            c if c.is_whitespace() && !in_quotes => {
                if !current.is_empty() {
                    tokens.push(current.clone());
                    current.clear();
                }
            }
            _ => {
                current.push(c);
            }
        }
    }

    if !current.is_empty() {
        tokens.push(current);
    }

    tokens
}
