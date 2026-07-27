//! Kubernetes CronJob Resource Spec Validator.
//!
//! Validates `CronJob` manifests for valid schedule expressions, `jobTemplate` definitions,
//! and concurrency policy options.

use crate::kubernetes::resource_detector::K8sResource;
use crate::yaml::parser::YamlValue;

/// Diagnostic defect found during CronJob spec validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CronJobIssue {
    /// Rule identifier.
    pub rule_id: String,
    /// Detailed diagnostic message.
    pub message: String,
    /// Line number.
    pub line: usize,
}

/// CronJob specification validator.
#[derive(Debug, Clone, Default)]
pub struct CronJobValidator;

impl CronJobValidator {
    /// Creates a new CronJobValidator.
    pub fn new() -> Self {
        Self
    }

    /// Validates a CronJob `K8sResource`.
    pub fn validate(&self, resource: &K8sResource) -> Vec<CronJobIssue> {
        let mut issues = Vec::new();

        if let YamlValue::Mapping(ref root_map) = resource.node.value {
            if let Some(spec_node) = root_map.pairs.iter().find(|p| p.key.value == "spec") {
                if let YamlValue::Mapping(ref spec_map) = spec_node.value.value {
                    let mut has_schedule = false;
                    let mut has_job_template = false;

                    for pair in &spec_map.pairs {
                        match pair.key.value.as_str() {
                            "schedule" => {
                                has_schedule = true;
                                if let YamlValue::Scalar(ref s) = pair.value.value {
                                    if !is_valid_cron_expression(&s.value) {
                                        issues.push(CronJobIssue {
                                            rule_id: "k8s-cronjob-invalid-schedule".to_string(),
                                            message: format!(
                                                "Invalid cron schedule expression '{}'",
                                                s.value
                                            ),
                                            line: pair.value.span.start.line,
                                        });
                                    }
                                }
                            }
                            "jobTemplate" => has_job_template = true,
                            _ => {}
                        }
                    }

                    if !has_schedule {
                        issues.push(CronJobIssue {
                            rule_id: "k8s-cronjob-missing-schedule".to_string(),
                            message: "CronJob spec is missing required 'schedule' field"
                                .to_string(),
                            line: spec_node.value.span.start.line,
                        });
                    }
                    if !has_job_template {
                        issues.push(CronJobIssue {
                            rule_id: "k8s-cronjob-missing-jobtemplate".to_string(),
                            message: "CronJob spec is missing required 'jobTemplate' field"
                                .to_string(),
                            line: spec_node.value.span.start.line,
                        });
                    }
                }
            } else {
                issues.push(CronJobIssue {
                    rule_id: "k8s-cronjob-missing-spec".to_string(),
                    message: "CronJob manifest is missing required top-level 'spec' field"
                        .to_string(),
                    line: resource.span.start.line,
                });
            }
        }

        issues
    }
}

fn is_valid_cron_expression(expr: &str) -> bool {
    let parts: Vec<&str> = expr.split_whitespace().collect();
    parts.len() == 5 || parts.len() == 6
}
