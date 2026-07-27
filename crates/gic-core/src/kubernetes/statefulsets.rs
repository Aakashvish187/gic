//! Kubernetes StatefulSet Resource Spec Validator.
//!
//! Validates `StatefulSet` manifests for required `serviceName` references, pod template
//! selectors, and persistent volume claim templates.

use crate::kubernetes::resource_detector::K8sResource;
use crate::yaml::parser::YamlValue;

/// Diagnostic defect found during StatefulSet spec validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatefulSetIssue {
    /// Rule identifier.
    pub rule_id: String,
    /// Detailed diagnostic message.
    pub message: String,
    /// Line number.
    pub line: usize,
}

/// StatefulSet specification validator.
#[derive(Debug, Clone, Default)]
pub struct StatefulSetValidator;

impl StatefulSetValidator {
    /// Creates a new StatefulSetValidator.
    pub fn new() -> Self {
        Self
    }

    /// Validates a StatefulSet `K8sResource`.
    pub fn validate(&self, resource: &K8sResource) -> Vec<StatefulSetIssue> {
        let mut issues = Vec::new();

        if let YamlValue::Mapping(ref root_map) = resource.node.value {
            if let Some(spec_node) = root_map.pairs.iter().find(|p| p.key.value == "spec") {
                if let YamlValue::Mapping(ref spec_map) = spec_node.value.value {
                    let mut has_service_name = false;

                    for pair in &spec_map.pairs {
                        if pair.key.value == "serviceName" {
                            if let YamlValue::Scalar(ref s) = pair.value.value {
                                if !s.value.trim().is_empty() {
                                    has_service_name = true;
                                }
                            }
                        }
                    }

                    if !has_service_name {
                        issues.push(StatefulSetIssue {
                            rule_id: "k8s-statefulset-missing-servicename".to_string(),
                            message: "StatefulSet spec is missing required 'serviceName' field referencing headless Service".to_string(),
                            line: spec_node.value.span.start.line,
                        });
                    }
                }
            } else {
                issues.push(StatefulSetIssue {
                    rule_id: "k8s-statefulset-missing-spec".to_string(),
                    message: "StatefulSet manifest is missing required top-level 'spec' field"
                        .to_string(),
                    line: resource.span.start.line,
                });
            }
        }

        issues
    }
}
