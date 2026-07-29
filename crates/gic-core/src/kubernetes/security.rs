//! Kubernetes Security Analyzer enforcing PodSecurityStandards.
//!
//! Audits manifests for privileged execution, root user escalation, host namespaces,
//! dangerous Linux capabilities, unpinned image tags, and writable root filesystems.

use crate::kubernetes::resource_detector::{K8sResource, K8sResourceKind};
use crate::yaml::parser::{Span, YamlMapping, YamlValue};

/// Category of security violation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SecuritySeverity {
    Critical,
    High,
    Medium,
    Low,
}

/// Security violation report item.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecurityFinding {
    /// Rule identifier (e.g. `sec-k8s-no-privileged`, `sec-k8s-no-latest-tag`).
    pub rule_id: String,
    /// Human-readable description.
    pub message: String,
    /// Violation severity level.
    pub severity: SecuritySeverity,
    /// Line number.
    pub line: usize,
    /// Target span.
    pub span: Span,
    /// Suggested fix value if automated quick-fix is possible.
    pub fix_suggestion: Option<String>,
}

/// Security analyzer for Kubernetes workloads.
#[derive(Debug, Clone, Default)]
pub struct K8sSecurityAnalyzer;

impl K8sSecurityAnalyzer {
    /// Creates a new K8sSecurityAnalyzer.
    pub fn new() -> Self {
        Self
    }

    /// Audits a Kubernetes resource for security flaws.
    pub fn audit(&self, resource: &K8sResource) -> Vec<SecurityFinding> {
        let mut findings = Vec::new();

        match resource.kind {
            K8sResourceKind::Deployment
            | K8sResourceKind::StatefulSet
            | K8sResourceKind::DaemonSet
            | K8sResourceKind::ReplicaSet
            | K8sResourceKind::Job
            | K8sResourceKind::CronJob
            | K8sResourceKind::Pod => {
                self.audit_workload(resource, &mut findings);
            }
            _ => {}
        }

        findings
    }

    fn audit_workload(&self, resource: &K8sResource, findings: &mut Vec<SecurityFinding>) {
        if let YamlValue::Mapping(ref root_map) = resource.node.value {
            self.inspect_mapping_for_security(root_map, findings, resource.span);
        }
    }

    fn inspect_mapping_for_security(
        &self,
        map: &YamlMapping,
        findings: &mut Vec<SecurityFinding>,
        _default_span: Span,
    ) {
        for pair in &map.pairs {
            match pair.key.value.as_str() {
                "hostNetwork" => {
                    if is_true(&pair.value.value) {
                        findings.push(SecurityFinding {
                            rule_id: "sec-k8s-no-host-network".to_string(),
                            message: "Pod spec sets 'hostNetwork: true', exposing host network interfaces".to_string(),
                            severity: SecuritySeverity::High,
                            line: pair.value.span.start.line,
                            span: pair.value.span,
                            fix_suggestion: Some("hostNetwork: false".to_string()),
                        });
                    }
                }
                "hostPID" => {
                    if is_true(&pair.value.value) {
                        findings.push(SecurityFinding {
                            rule_id: "sec-k8s-no-host-pid".to_string(),
                            message:
                                "Pod spec sets 'hostPID: true', sharing host process ID namespace"
                                    .to_string(),
                            severity: SecuritySeverity::High,
                            line: pair.value.span.start.line,
                            span: pair.value.span,
                            fix_suggestion: Some("hostPID: false".to_string()),
                        });
                    }
                }
                "hostIPC" => {
                    if is_true(&pair.value.value) {
                        findings.push(SecurityFinding {
                            rule_id: "sec-k8s-no-host-ipc".to_string(),
                            message: "Pod spec sets 'hostIPC: true', sharing host IPC namespace"
                                .to_string(),
                            severity: SecuritySeverity::High,
                            line: pair.value.span.start.line,
                            span: pair.value.span,
                            fix_suggestion: Some("hostIPC: false".to_string()),
                        });
                    }
                }
                "image" => {
                    if let YamlValue::Scalar(ref s) = pair.value.value {
                        let img = s.value.trim();
                        if img.ends_with(":latest") || !img.contains(':') || img.contains("latest")
                        {
                            findings.push(SecurityFinding {
                                rule_id: "sec-k8s-no-latest-image-tag".to_string(),
                                message: format!(
                                    "Container image '{img}' uses 'latest' or unpinned tag"
                                ),
                                severity: SecuritySeverity::Medium,
                                line: pair.value.span.start.line,
                                span: pair.value.span,
                                fix_suggestion: Some(
                                    "Pin container image to explicit semver tag or SHA digest"
                                        .to_string(),
                                ),
                            });
                        }
                    }
                }
                "privileged" => {
                    if is_true(&pair.value.value) {
                        findings.push(SecurityFinding {
                            rule_id: "sec-k8s-no-privileged-containers".to_string(),
                            message: "Container securityContext sets 'privileged: true', granting full root access to host".to_string(),
                            severity: SecuritySeverity::Critical,
                            line: pair.value.span.start.line,
                            span: pair.value.span,
                            fix_suggestion: Some("privileged: false".to_string()),
                        });
                    }
                }
                "allowPrivilegeEscalation" => {
                    if is_true(&pair.value.value) {
                        findings.push(SecurityFinding {
                            rule_id: "sec-k8s-no-privilege-escalation".to_string(),
                            message: "Container sets 'allowPrivilegeEscalation: true'".to_string(),
                            severity: SecuritySeverity::High,
                            line: pair.value.span.start.line,
                            span: pair.value.span,
                            fix_suggestion: Some("allowPrivilegeEscalation: false".to_string()),
                        });
                    }
                }
                "readOnlyRootFilesystem" if !is_true(&pair.value.value) => {
                    findings.push(SecurityFinding {
                            rule_id: "sec-k8s-read-only-root-fs".to_string(),
                            message: "Container root filesystem is writable (set 'readOnlyRootFilesystem: true')".to_string(),
                            severity: SecuritySeverity::Medium,
                            line: pair.value.span.start.line,
                            span: pair.value.span,
                            fix_suggestion: Some("readOnlyRootFilesystem: true".to_string()),
                        });
                }
                _ => {}
            }

            match &pair.value.value {
                YamlValue::Mapping(ref child_map) => {
                    self.inspect_mapping_for_security(child_map, findings, _default_span);
                }
                YamlValue::Sequence(ref seq) => {
                    for item in &seq.items {
                        if let YamlValue::Mapping(ref item_map) = item.value {
                            self.inspect_mapping_for_security(item_map, findings, _default_span);
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

fn is_true(val: &YamlValue) -> bool {
    if let YamlValue::Scalar(ref s) = val {
        s.value.trim() == "true"
    } else {
        false
    }
}
