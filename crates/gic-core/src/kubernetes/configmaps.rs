//! Kubernetes ConfigMap Resource Spec Validator.
//!
//! Validates `ConfigMap` manifests for `data` and `binaryData` dictionary formats.

use crate::kubernetes::resource_detector::K8sResource;
use crate::yaml::parser::YamlValue;

/// Diagnostic defect found during ConfigMap validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigMapIssue {
    /// Rule identifier.
    pub rule_id: String,
    /// Detailed diagnostic message.
    pub message: String,
    /// Line number.
    pub line: usize,
}

/// ConfigMap specification validator.
#[derive(Debug, Clone, Default)]
pub struct ConfigMapValidator;

impl ConfigMapValidator {
    /// Creates a new ConfigMapValidator.
    pub fn new() -> Self {
        Self
    }

    /// Validates a ConfigMap `K8sResource`.
    pub fn validate(&self, resource: &K8sResource) -> Vec<ConfigMapIssue> {
        let mut issues = Vec::new();

        if let YamlValue::Mapping(ref root_map) = resource.node.value {
            let mut has_data = false;
            let mut has_binary_data = false;

            for pair in &root_map.pairs {
                match pair.key.value.as_str() {
                    "data" => has_data = true,
                    "binaryData" => has_binary_data = true,
                    _ => {}
                }
            }

            if !has_data && !has_binary_data {
                issues.push(ConfigMapIssue {
                    rule_id: "k8s-configmap-empty".to_string(),
                    message: "ConfigMap contains no 'data' or 'binaryData' payload entries"
                        .to_string(),
                    line: resource.span.start.line,
                });
            }
        }

        issues
    }
}
