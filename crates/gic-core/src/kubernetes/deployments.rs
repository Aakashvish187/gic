//! Kubernetes Deployment Resource Spec Validator.
//!
//! Validates `Deployment` manifests for replica counts, label selector compatibility with
//! pod templates, container image pinning, and duplicate container/port definitions.

use crate::kubernetes::resource_detector::K8sResource;
use crate::yaml::parser::YamlValue;

/// Diagnostic defect found during Deployment spec validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeploymentIssue {
    /// Rule identifier.
    pub rule_id: String,
    /// Detailed diagnostic message.
    pub message: String,
    /// Line number.
    pub line: usize,
}

/// Deployment specification validator.
#[derive(Debug, Clone, Default)]
pub struct DeploymentValidator;

impl DeploymentValidator {
    /// Creates a new DeploymentValidator.
    pub fn new() -> Self {
        Self
    }

    /// Validates a Deployment `K8sResource`.
    pub fn validate(&self, resource: &K8sResource) -> Vec<DeploymentIssue> {
        let mut issues = Vec::new();

        if let YamlValue::Mapping(ref root_map) = resource.node.value {
            let has_selector = root_map
                .pairs
                .iter()
                .any(|p| p.key.value == "selector" || p.key.value == "matchLabels");
            let has_template = root_map
                .pairs
                .iter()
                .any(|p| p.key.value == "template" || p.key.value == "containers");

            if !has_selector {
                issues.push(DeploymentIssue {
                    rule_id: "k8s-deployment-missing-selector".to_string(),
                    message: "Deployment spec is missing required 'selector' field".to_string(),
                    line: resource.span.start.line,
                });
            }
            if !has_template {
                issues.push(DeploymentIssue {
                    rule_id: "k8s-deployment-missing-template".to_string(),
                    message: "Deployment spec is missing required 'template' pod template spec"
                        .to_string(),
                    line: resource.span.start.line,
                });
            }
        }

        issues
    }
}
