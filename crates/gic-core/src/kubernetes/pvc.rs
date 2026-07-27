//! Kubernetes PersistentVolumeClaim Resource Spec Validator.
//!
//! Validates `PersistentVolumeClaim` manifests for `accessModes` arrays and storage request limits.

use crate::kubernetes::resource_detector::K8sResource;
use crate::yaml::parser::YamlValue;

/// Diagnostic defect found during PVC validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PvcIssue {
    /// Rule identifier.
    pub rule_id: String,
    /// Detailed diagnostic message.
    pub message: String,
    /// Line number.
    pub line: usize,
}

/// PVC specification validator.
#[derive(Debug, Clone, Default)]
pub struct PvcValidator;

impl PvcValidator {
    /// Creates a new PvcValidator.
    pub fn new() -> Self {
        Self
    }

    /// Validates a PersistentVolumeClaim `K8sResource`.
    pub fn validate(&self, resource: &K8sResource) -> Vec<PvcIssue> {
        let mut issues = Vec::new();

        if let YamlValue::Mapping(ref root_map) = resource.node.value {
            if let Some(spec_node) = root_map.pairs.iter().find(|p| p.key.value == "spec") {
                if let YamlValue::Mapping(ref spec_map) = spec_node.value.value {
                    let mut has_access_modes = false;
                    let mut has_resources = false;

                    for pair in &spec_map.pairs {
                        match pair.key.value.as_str() {
                            "accessModes" => has_access_modes = true,
                            "resources" => has_resources = true,
                            _ => {}
                        }
                    }

                    if !has_access_modes {
                        issues.push(PvcIssue {
                            rule_id: "k8s-pvc-missing-accessmodes".to_string(),
                            message:
                                "PersistentVolumeClaim spec is missing required 'accessModes' list"
                                    .to_string(),
                            line: spec_node.value.span.start.line,
                        });
                    }
                    if !has_resources {
                        issues.push(PvcIssue {
                            rule_id: "k8s-pvc-missing-resources".to_string(),
                            message: "PersistentVolumeClaim spec is missing required 'resources.requests.storage' field".to_string(),
                            line: spec_node.value.span.start.line,
                        });
                    }
                }
            } else {
                issues.push(PvcIssue {
                    rule_id: "k8s-pvc-missing-spec".to_string(),
                    message:
                        "PersistentVolumeClaim manifest is missing required top-level 'spec' field"
                            .to_string(),
                    line: resource.span.start.line,
                });
            }
        }

        issues
    }
}
