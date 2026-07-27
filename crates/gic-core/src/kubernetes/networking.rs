//! Kubernetes NetworkPolicy and Traffic Routing Validator.

use crate::kubernetes::resource_detector::K8sResource;
use crate::yaml::parser::YamlValue;

/// Diagnostic defect found during NetworkPolicy validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetworkPolicyIssue {
    pub rule_id: String,
    pub message: String,
    pub line: usize,
}

/// NetworkPolicy validator.
#[derive(Debug, Clone, Default)]
pub struct NetworkPolicyValidator;

impl NetworkPolicyValidator {
    pub fn new() -> Self {
        Self
    }

    pub fn validate(&self, resource: &K8sResource) -> Vec<NetworkPolicyIssue> {
        let mut issues = Vec::new();

        if resource.raw_kind == "NetworkPolicy" {
            if let YamlValue::Mapping(ref root_map) = resource.node.value {
                if let Some(spec_node) = root_map.pairs.iter().find(|p| p.key.value == "spec") {
                    if let YamlValue::Mapping(ref spec_map) = spec_node.value.value {
                        let has_pod_selector =
                            spec_map.pairs.iter().any(|p| p.key.value == "podSelector");
                        if !has_pod_selector {
                            issues.push(NetworkPolicyIssue {
                                rule_id: "k8s-netpol-missing-podselector".to_string(),
                                message:
                                    "NetworkPolicy spec is missing required 'podSelector' field"
                                        .to_string(),
                                line: spec_node.value.span.start.line,
                            });
                        }
                    }
                }
            }
        }

        issues
    }
}
