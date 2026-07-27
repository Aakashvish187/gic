//! Kubernetes StorageClass and PersistentVolume Validator.

use crate::kubernetes::resource_detector::K8sResource;
use crate::yaml::parser::YamlValue;

/// Diagnostic defect found during Storage validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StorageIssue {
    pub rule_id: String,
    pub message: String,
    pub line: usize,
}

/// StorageClass specification validator.
#[derive(Debug, Clone, Default)]
pub struct StorageValidator;

impl StorageValidator {
    pub fn new() -> Self {
        Self
    }

    pub fn validate(&self, resource: &K8sResource) -> Vec<StorageIssue> {
        let mut issues = Vec::new();

        if resource.raw_kind == "StorageClass" {
            if let YamlValue::Mapping(ref root_map) = resource.node.value {
                let has_provisioner = root_map.pairs.iter().any(|p| p.key.value == "provisioner");
                if !has_provisioner {
                    issues.push(StorageIssue {
                        rule_id: "k8s-storageclass-missing-provisioner".to_string(),
                        message: "StorageClass manifest is missing required 'provisioner' field"
                            .to_string(),
                        line: resource.span.start.line,
                    });
                }
            }
        }

        issues
    }
}
