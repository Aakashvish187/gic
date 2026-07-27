//! Kubernetes DaemonSet Resource Spec Validator.
//!
//! Validates `DaemonSet` manifests for required pod template selectors and container definitions.

use crate::kubernetes::resource_detector::K8sResource;
use crate::yaml::parser::YamlValue;

/// Diagnostic defect found during DaemonSet spec validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DaemonSetIssue {
    /// Rule identifier.
    pub rule_id: String,
    /// Detailed diagnostic message.
    pub message: String,
    /// Line number.
    pub line: usize,
}

/// DaemonSet specification validator.
#[derive(Debug, Clone, Default)]
pub struct DaemonSetValidator;

impl DaemonSetValidator {
    /// Creates a new DaemonSetValidator.
    pub fn new() -> Self {
        Self
    }

    /// Validates a DaemonSet `K8sResource`.
    pub fn validate(&self, resource: &K8sResource) -> Vec<DaemonSetIssue> {
        let mut issues = Vec::new();

        if let YamlValue::Mapping(ref root_map) = resource.node.value {
            if let Some(spec_node) = root_map.pairs.iter().find(|p| p.key.value == "spec") {
                if let YamlValue::Mapping(ref spec_map) = spec_node.value.value {
                    let mut has_template = false;
                    let mut has_selector = false;

                    for pair in &spec_map.pairs {
                        match pair.key.value.as_str() {
                            "template" => has_template = true,
                            "selector" => has_selector = true,
                            _ => {}
                        }
                    }

                    if !has_selector {
                        issues.push(DaemonSetIssue {
                            rule_id: "k8s-daemonset-missing-selector".to_string(),
                            message: "DaemonSet spec is missing required 'selector' field"
                                .to_string(),
                            line: spec_node.value.span.start.line,
                        });
                    }
                    if !has_template {
                        issues.push(DaemonSetIssue {
                            rule_id: "k8s-daemonset-missing-template".to_string(),
                            message: "DaemonSet spec is missing required 'template' field"
                                .to_string(),
                            line: spec_node.value.span.start.line,
                        });
                    }
                }
            } else {
                issues.push(DaemonSetIssue {
                    rule_id: "k8s-daemonset-missing-spec".to_string(),
                    message: "DaemonSet manifest is missing required top-level 'spec' field"
                        .to_string(),
                    line: resource.span.start.line,
                });
            }
        }

        issues
    }
}
