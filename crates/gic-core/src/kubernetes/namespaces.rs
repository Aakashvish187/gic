//! Kubernetes Namespace Scope and Reference Validator.

use crate::kubernetes::resource_detector::K8sResource;

/// Diagnostic defect found during Namespace validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NamespaceIssue {
    pub rule_id: String,
    pub message: String,
    pub line: usize,
}

/// Namespace validator.
#[derive(Debug, Clone, Default)]
pub struct NamespaceValidator;

impl NamespaceValidator {
    pub fn new() -> Self {
        Self
    }

    pub fn validate(&self, resource: &K8sResource) -> Vec<NamespaceIssue> {
        let mut issues = Vec::new();

        if resource.metadata.name.is_empty() {
            issues.push(NamespaceIssue {
                rule_id: "k8s-missing-name".to_string(),
                message: format!("{} is missing required 'metadata.name'", resource.kind),
                line: resource.span.start.line,
            });
        }

        issues
    }
}
