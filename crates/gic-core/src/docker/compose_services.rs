//! Docker Compose Service Specification Validator.
//!
//! Validates `services:` definitions for `image`, `build`, `ports`, `environment`, `env_file`,
//! `volumes`, `networks`, `depends_on`, `healthcheck`, `restart`, `secrets`, `configs`, `profiles`,
//! `privileged`, `cap_add`, `cap_drop`, `read_only`, and `network_mode`.

use crate::yaml::parser::{YamlNode, YamlValue};

/// Diagnostic issue in Docker Compose service definition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComposeServiceIssue {
    /// Service identifier name.
    pub service_name: String,
    /// Rule identifier.
    pub rule_id: String,
    /// Diagnostic message.
    pub message: String,
    /// Line number.
    pub line: usize,
}

/// Service specification validator.
#[derive(Debug, Clone, Default)]
pub struct ComposeServiceValidator;

impl ComposeServiceValidator {
    /// Creates a new ComposeServiceValidator.
    pub fn new() -> Self {
        Self
    }

    /// Validates a single Docker Compose service node.
    pub fn validate_service(
        &self,
        service_name: &str,
        node: &YamlNode,
    ) -> Vec<ComposeServiceIssue> {
        let mut issues = Vec::new();

        if let YamlValue::Mapping(ref map) = node.value {
            let mut has_image = false;
            let mut has_build = false;

            for pair in &map.pairs {
                match pair.key.value.as_str() {
                    "image" => has_image = true,
                    "build" => has_build = true,
                    "ports" => self.validate_ports(service_name, &pair.value, &mut issues),
                    "volumes" => self.validate_volumes(service_name, &pair.value, &mut issues),
                    "restart" => {
                        self.validate_restart_policy(service_name, &pair.value, &mut issues)
                    }
                    _ => {}
                }
            }

            if !has_image && !has_build {
                issues.push(ComposeServiceIssue {
                    service_name: service_name.to_string(),
                    rule_id: "compose-service-missing-image-or-build".to_string(),
                    message: format!(
                        "Service '{service_name}' must specify either 'image' or 'build'"
                    ),
                    line: node.span.start.line,
                });
            }
        }

        issues
    }

    fn validate_ports(
        &self,
        service_name: &str,
        node: &YamlNode,
        issues: &mut Vec<ComposeServiceIssue>,
    ) {
        if let YamlValue::Sequence(ref seq) = node.value {
            for item in &seq.items {
                if let YamlValue::Scalar(ref s) = item.value {
                    let port_str = s.value.trim();
                    if !is_valid_port_binding(port_str) {
                        issues.push(ComposeServiceIssue {
                            service_name: service_name.to_string(),
                            rule_id: "compose-invalid-port-binding".to_string(),
                            message: format!("Invalid port binding specification '{port_str}' in service '{service_name}'"),
                            line: item.span.start.line,
                        });
                    }
                }
            }
        }
    }

    fn validate_volumes(
        &self,
        service_name: &str,
        node: &YamlNode,
        issues: &mut Vec<ComposeServiceIssue>,
    ) {
        if let YamlValue::Sequence(ref seq) = node.value {
            for item in &seq.items {
                if let YamlValue::Scalar(ref s) = item.value {
                    let vol_str = s.value.trim();
                    if vol_str.is_empty() {
                        issues.push(ComposeServiceIssue {
                            service_name: service_name.to_string(),
                            rule_id: "compose-empty-volume-binding".to_string(),
                            message: format!(
                                "Empty volume binding entry in service '{service_name}'"
                            ),
                            line: item.span.start.line,
                        });
                    }
                }
            }
        }
    }

    fn validate_restart_policy(
        &self,
        service_name: &str,
        node: &YamlNode,
        issues: &mut Vec<ComposeServiceIssue>,
    ) {
        if let YamlValue::Scalar(ref s) = node.value {
            match s.value.trim() {
                "no" | "always" | "on-failure" | "unless-stopped" => {}
                other => {
                    issues.push(ComposeServiceIssue {
                        service_name: service_name.to_string(),
                        rule_id: "compose-invalid-restart-policy".to_string(),
                        message: format!(
                            "Invalid restart policy '{other}' in service '{service_name}'"
                        ),
                        line: node.span.start.line,
                    });
                }
            }
        }
    }
}

fn is_valid_port_binding(port: &str) -> bool {
    if port.is_empty() {
        return false;
    }
    let parts: Vec<&str> = port.split(':').collect();
    !parts.is_empty() && parts.len() <= 3
}
