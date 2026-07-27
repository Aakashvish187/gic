//! Docker Compose Top-Level Network Specification Validator.

use crate::yaml::parser::{YamlNode, YamlValue};

/// Diagnostic issue in Docker Compose network definition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComposeNetworkIssue {
    pub network_name: String,
    pub rule_id: String,
    pub message: String,
    pub line: usize,
}

/// Network specification validator.
#[derive(Debug, Clone, Default)]
pub struct ComposeNetworkValidator;

impl ComposeNetworkValidator {
    pub fn new() -> Self {
        Self
    }

    pub fn validate_network(
        &self,
        network_name: &str,
        node: &YamlNode,
    ) -> Vec<ComposeNetworkIssue> {
        let mut issues = Vec::new();

        if let YamlValue::Mapping(ref map) = node.value {
            let mut driver = None;
            for pair in &map.pairs {
                if pair.key.value == "driver" {
                    if let YamlValue::Scalar(ref s) = pair.value.value {
                        driver = Some(s.value.clone());
                    }
                }
            }

            if let Some(d) = driver {
                match d.as_str() {
                    "bridge" | "host" | "overlay" | "macvlan" | "none" => {}
                    other => {
                        issues.push(ComposeNetworkIssue {
                            network_name: network_name.to_string(),
                            rule_id: "compose-invalid-network-driver".to_string(),
                            message: format!(
                                "Unknown network driver '{other}' in network '{network_name}'"
                            ),
                            line: node.span.start.line,
                        });
                    }
                }
            }
        }

        issues
    }
}
