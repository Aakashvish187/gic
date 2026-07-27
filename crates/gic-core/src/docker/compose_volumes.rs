//! Docker Compose Top-Level Volume Specification Validator.

use crate::yaml::parser::{YamlNode, YamlValue};

/// Diagnostic issue in Docker Compose volume definition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComposeVolumeIssue {
    pub volume_name: String,
    pub rule_id: String,
    pub message: String,
    pub line: usize,
}

/// Volume specification validator.
#[derive(Debug, Clone, Default)]
pub struct ComposeVolumeValidator;

impl ComposeVolumeValidator {
    pub fn new() -> Self {
        Self
    }

    pub fn validate_volume(&self, volume_name: &str, node: &YamlNode) -> Vec<ComposeVolumeIssue> {
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
                if d.trim().is_empty() {
                    issues.push(ComposeVolumeIssue {
                        volume_name: volume_name.to_string(),
                        rule_id: "compose-empty-volume-driver".to_string(),
                        message: format!("Volume '{volume_name}' driver cannot be empty"),
                        line: node.span.start.line,
                    });
                }
            }
        }

        issues
    }
}
