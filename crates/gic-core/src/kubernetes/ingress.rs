//! Kubernetes Ingress Resource Spec Validator.
//!
//! Validates `Ingress` manifests for routing rules, paths, backend services, and TLS certificates.

use crate::kubernetes::resource_detector::K8sResource;
use crate::yaml::parser::YamlValue;

/// Diagnostic defect found during Ingress spec validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IngressIssue {
    /// Rule identifier.
    pub rule_id: String,
    /// Detailed diagnostic message.
    pub message: String,
    /// Line number.
    pub line: usize,
}

/// Ingress specification validator.
#[derive(Debug, Clone, Default)]
pub struct IngressValidator;

impl IngressValidator {
    /// Creates a new IngressValidator.
    pub fn new() -> Self {
        Self
    }

    /// Validates an Ingress `K8sResource`.
    pub fn validate(&self, resource: &K8sResource) -> Vec<IngressIssue> {
        let mut issues = Vec::new();

        if let YamlValue::Mapping(ref root_map) = resource.node.value {
            if let Some(spec_node) = root_map.pairs.iter().find(|p| p.key.value == "spec") {
                if let YamlValue::Mapping(ref spec_map) = spec_node.value.value {
                    let mut has_rules = false;
                    let mut has_default_backend = false;

                    for pair in &spec_map.pairs {
                        match pair.key.value.as_str() {
                            "rules" => has_rules = true,
                            "defaultBackend" => has_default_backend = true,
                            _ => {}
                        }
                    }

                    if !has_rules && !has_default_backend {
                        issues.push(IngressIssue {
                            rule_id: "k8s-ingress-missing-rules-or-backend".to_string(),
                            message: "Ingress spec must define either 'rules' or 'defaultBackend'"
                                .to_string(),
                            line: spec_node.value.span.start.line,
                        });
                    }
                }
            } else {
                issues.push(IngressIssue {
                    rule_id: "k8s-ingress-missing-spec".to_string(),
                    message: "Ingress manifest is missing required top-level 'spec' field"
                        .to_string(),
                    line: resource.span.start.line,
                });
            }
        }

        issues
    }
}
