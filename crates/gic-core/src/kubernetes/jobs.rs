//! Kubernetes Job Resource Spec Validator.
//!
//! Validates `Job` manifests for `restartPolicy` compliance (must be `Never` or `OnFailure`).

use crate::kubernetes::resource_detector::K8sResource;
use crate::yaml::parser::YamlValue;

/// Diagnostic defect found during Job spec validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JobIssue {
    /// Rule identifier.
    pub rule_id: String,
    /// Detailed diagnostic message.
    pub message: String,
    /// Line number.
    pub line: usize,
}

/// Job specification validator.
#[derive(Debug, Clone, Default)]
pub struct JobValidator;

impl JobValidator {
    /// Creates a new JobValidator.
    pub fn new() -> Self {
        Self
    }

    /// Validates a Job `K8sResource`.
    pub fn validate(&self, resource: &K8sResource) -> Vec<JobIssue> {
        let mut issues = Vec::new();

        if let YamlValue::Mapping(ref root_map) = resource.node.value {
            let mut restart_policy = None;
            let mut line = resource.span.start.line;

            for pair in &root_map.pairs {
                if pair.key.value == "restartPolicy" {
                    if let YamlValue::Scalar(ref s) = pair.value.value {
                        restart_policy = Some(s.value.clone());
                        line = pair.value.span.start.line;
                    }
                }
            }

            match restart_policy.as_deref() {
                Some("Never") | Some("OnFailure") => {}
                Some("Always") => {
                    issues.push(JobIssue {
                        rule_id: "k8s-job-invalid-restart-policy".to_string(),
                        message: "Job container restartPolicy cannot be 'Always' (must be 'Never' or 'OnFailure')".to_string(),
                        line,
                    });
                }
                None => {
                    issues.push(JobIssue {
                        rule_id: "k8s-job-missing-restart-policy".to_string(),
                        message: "Job pod template spec is missing required 'restartPolicy' field (must be 'Never' or 'OnFailure')".to_string(),
                        line,
                    });
                }
                _ => {}
            }
        }

        issues
    }
}
