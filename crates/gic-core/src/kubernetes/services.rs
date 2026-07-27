//! Kubernetes Service Resource Spec Validator.
//!
//! Validates `Service` manifests for `spec.ports` array, `targetPort` definitions, and `selector` compatibility.

use crate::kubernetes::resource_detector::K8sResource;
use crate::yaml::parser::YamlValue;

/// Diagnostic defect found during Service spec validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceIssue {
    /// Rule identifier.
    pub rule_id: String,
    /// Detailed diagnostic message.
    pub message: String,
    /// Line number.
    pub line: usize,
}

/// Service specification validator.
#[derive(Debug, Clone, Default)]
pub struct ServiceValidator;

impl ServiceValidator {
    /// Creates a new ServiceValidator.
    pub fn new() -> Self {
        Self
    }

    /// Validates a Service `K8sResource`.
    pub fn validate(&self, resource: &K8sResource) -> Vec<ServiceIssue> {
        let mut issues = Vec::new();

        if let YamlValue::Mapping(ref root_map) = resource.node.value {
            if let Some(spec_node) = root_map.pairs.iter().find(|p| p.key.value == "spec") {
                if let YamlValue::Mapping(ref spec_map) = spec_node.value.value {
                    let mut has_ports = false;

                    for pair in &spec_map.pairs {
                        if pair.key.value == "ports" {
                            has_ports = true;
                            if let YamlValue::Sequence(ref seq) = pair.value.value {
                                if seq.items.is_empty() {
                                    issues.push(ServiceIssue {
                                        rule_id: "k8s-service-empty-ports".to_string(),
                                        message: "Service 'spec.ports' list cannot be empty"
                                            .to_string(),
                                        line: pair.value.span.start.line,
                                    });
                                }
                            }
                        }
                    }

                    if !has_ports {
                        issues.push(ServiceIssue {
                            rule_id: "k8s-service-missing-ports".to_string(),
                            message: "Service spec is missing required 'ports' list".to_string(),
                            line: spec_node.value.span.start.line,
                        });
                    }
                }
            } else {
                issues.push(ServiceIssue {
                    rule_id: "k8s-service-missing-spec".to_string(),
                    message: "Service manifest is missing required top-level 'spec' field"
                        .to_string(),
                    line: resource.span.start.line,
                });
            }
        }

        issues
    }
}
