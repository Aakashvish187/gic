//! Central Linux & Bash Validation Engine.

use crate::linux::commands::CommandRegistry;
use crate::linux::cron::CronAnalyzer;
use crate::linux::environment::EnvironmentAnalyzer;
use crate::linux::filesystem::FilesystemAnalyzer;
use crate::linux::groups::GroupsAnalyzer;
use crate::linux::networking::NetworkAnalyzer;
use crate::linux::packages::PackageAnalyzer;
use crate::linux::permissions::PermissionsAnalyzer;
use crate::linux::security::SecurityAnalyzer;
use crate::linux::shell::BashParser;
use crate::linux::ssh::SshAnalyzer;
use crate::linux::systemd::SystemdAnalyzer;
use crate::linux::users::UsersAnalyzer;
use crate::linux::variables::VariableTracker;
use crate::yaml::parser::{Position, Span};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinuxDiagnostic {
    pub rule_id: String,
    pub message: String,
    pub span: Span,
    pub is_error: bool,
}

#[derive(Debug, Clone, Default)]
pub struct LinuxValidator {
    bash_parser: BashParser,
    command_registry: CommandRegistry,
    variable_tracker: VariableTracker,
    security_analyzer: SecurityAnalyzer,
    permissions_analyzer: PermissionsAnalyzer,
    package_analyzer: PackageAnalyzer,
    systemd_analyzer: SystemdAnalyzer,
    ssh_analyzer: SshAnalyzer,
    cron_analyzer: CronAnalyzer,
    env_analyzer: EnvironmentAnalyzer,
    fs_analyzer: FilesystemAnalyzer,
    users_analyzer: UsersAnalyzer,
    groups_analyzer: GroupsAnalyzer,
    network_analyzer: NetworkAnalyzer,
}

impl LinuxValidator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn validate_bash_script(&self, source: &str) -> Vec<LinuxDiagnostic> {
        let mut diags = Vec::new();
        match self.bash_parser.parse(source) {
            Ok(ast) => {
                if let Ok(cmds) = self.command_registry.validate_commands(&ast) {
                    diags.extend(cmds.into_iter().map(|d| LinuxDiagnostic {
                        rule_id: d.rule_id,
                        message: d.message,
                        span: d.span,
                        is_error: d.is_error,
                    }));
                }
                if let Ok(vars) = self.variable_tracker.analyze(&ast) {
                    diags.extend(vars.into_iter().map(|d| LinuxDiagnostic {
                        rule_id: d.rule_id,
                        message: d.message,
                        span: d.span,
                        is_error: d.is_error,
                    }));
                }
                if let Ok(secs) = self.security_analyzer.analyze_bash(&ast) {
                    diags.extend(secs.into_iter().map(|d| LinuxDiagnostic {
                        rule_id: d.rule_id,
                        message: d.message,
                        span: d.span,
                        is_error: d.is_error,
                    }));
                }
                if let Ok(perms) = self.permissions_analyzer.analyze(&ast) {
                    diags.extend(perms.into_iter().map(|d| LinuxDiagnostic {
                        rule_id: d.rule_id,
                        message: d.message,
                        span: d.span,
                        is_error: d.is_error,
                    }));
                }
                if let Ok(pkgs) = self.package_analyzer.analyze(&ast) {
                    diags.extend(pkgs.into_iter().map(|d| LinuxDiagnostic {
                        rule_id: d.rule_id,
                        message: d.message,
                        span: d.span,
                        is_error: d.is_error,
                    }));
                }
            }
            Err(e) => {
                diags.push(LinuxDiagnostic {
                    rule_id: "lin-syntax-error".to_string(),
                    message: e.to_string(),
                    span: Span::new(Position::new(1, 1, 0), Position::new(1, 1, 0)),
                    is_error: true,
                });
            }
        }
        diags
    }

    pub fn validate_systemd(&self, source: &str) -> Vec<LinuxDiagnostic> {
        self.systemd_analyzer
            .analyze(source)
            .unwrap_or_default()
            .into_iter()
            .map(|d| LinuxDiagnostic {
                rule_id: d.rule_id,
                message: d.message,
                span: d.span,
                is_error: d.is_error,
            })
            .collect()
    }

    pub fn validate_sshd(&self, source: &str) -> Vec<LinuxDiagnostic> {
        self.ssh_analyzer
            .analyze_sshd(source)
            .unwrap_or_default()
            .into_iter()
            .map(|d| LinuxDiagnostic {
                rule_id: d.rule_id,
                message: d.message,
                span: d.span,
                is_error: d.is_error,
            })
            .collect()
    }

    pub fn validate_cron(&self, source: &str) -> Vec<LinuxDiagnostic> {
        self.cron_analyzer
            .analyze(source)
            .unwrap_or_default()
            .into_iter()
            .map(|d| LinuxDiagnostic {
                rule_id: d.rule_id,
                message: d.message,
                span: d.span,
                is_error: d.is_error,
            })
            .collect()
    }

    pub fn validate_env(&self, source: &str) -> Vec<LinuxDiagnostic> {
        self.env_analyzer
            .analyze(source)
            .unwrap_or_default()
            .into_iter()
            .map(|d| LinuxDiagnostic {
                rule_id: d.rule_id,
                message: d.message,
                span: d.span,
                is_error: d.is_error,
            })
            .collect()
    }

    pub fn validate_fstab(&self, source: &str) -> Vec<LinuxDiagnostic> {
        self.fs_analyzer
            .analyze_fstab(source)
            .unwrap_or_default()
            .into_iter()
            .map(|d| LinuxDiagnostic {
                rule_id: d.rule_id,
                message: d.message,
                span: d.span,
                is_error: d.is_error,
            })
            .collect()
    }

    pub fn validate_passwd(&self, source: &str) -> Vec<LinuxDiagnostic> {
        self.users_analyzer
            .analyze_passwd(source)
            .unwrap_or_default()
            .into_iter()
            .map(|d| LinuxDiagnostic {
                rule_id: d.rule_id,
                message: d.message,
                span: d.span,
                is_error: d.is_error,
            })
            .collect()
    }

    pub fn validate_sudoers(&self, source: &str) -> Vec<LinuxDiagnostic> {
        self.users_analyzer
            .analyze_sudoers(source)
            .unwrap_or_default()
            .into_iter()
            .map(|d| LinuxDiagnostic {
                rule_id: d.rule_id,
                message: d.message,
                span: d.span,
                is_error: d.is_error,
            })
            .collect()
    }

    pub fn validate_group(&self, source: &str) -> Vec<LinuxDiagnostic> {
        self.groups_analyzer
            .analyze_group(source)
            .unwrap_or_default()
            .into_iter()
            .map(|d| LinuxDiagnostic {
                rule_id: d.rule_id,
                message: d.message,
                span: d.span,
                is_error: d.is_error,
            })
            .collect()
    }

    pub fn validate_hosts(&self, source: &str) -> Vec<LinuxDiagnostic> {
        self.network_analyzer
            .analyze_hosts(source)
            .unwrap_or_default()
            .into_iter()
            .map(|d| LinuxDiagnostic {
                rule_id: d.rule_id,
                message: d.message,
                span: d.span,
                is_error: d.is_error,
            })
            .collect()
    }
}
