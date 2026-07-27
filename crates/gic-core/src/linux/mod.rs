//! Linux & Bash Intelligence Engine for GIC.
//!
//! Provides static analysis for Bash/POSIX shell scripts and Linux configuration files.
//! Supported configurations: systemd, sshd, crontab, environment, fstab, passwd, group, sudoers, networking.

pub mod apparmor;
pub mod cache;
pub mod commands;
pub mod completion;
pub mod cron;
pub mod diagnostics;
pub mod engine;
pub mod environment;
pub mod errors;
pub mod filesystem;
pub mod firewall;
pub mod formatter;
pub mod groups;
pub mod hover;
pub mod logs;
pub mod networking;
pub mod packages;
pub mod permissions;
pub mod security;
pub mod selinux;
pub mod services;
pub mod shell;
pub mod ssh;
pub mod systemd;
pub mod users;
pub mod validator;
pub mod variables;

#[cfg(test)]
pub mod tests;

pub use cache::LinuxCache;
pub use completion::{CompletionKind, LinuxCompleter, LinuxCompletionItem};
pub use diagnostics::{convert_linux_diagnostic, convert_linux_diagnostics};
pub use engine::{LinuxEngine, LinuxEngineOptions};
pub use errors::{LinuxError, LinuxResult};
pub use formatter::LinuxFormatter;
pub use hover::{HoverDoc, LinuxHoverProvider};
pub use shell::{BashAST, BashParser, CommandInvocation, Shebang, ShellKind};
pub use validator::{LinuxDiagnostic, LinuxValidator};
