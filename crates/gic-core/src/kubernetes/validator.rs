//! Kubernetes Central Validation and Relationship Engine.
//!
//! Validates Kubernetes resources, apiVersions, manifest specifications, security contexts,
//! best practice guidelines, and cross-resource relationships (Service -> Pod, Ingress -> Service, PVC -> PV).

use crate::kubernetes::api_version::{ApiVersionStatus, K8sApiVersionEvaluator};
use crate::kubernetes::best_practices::K8sBestPracticesAnalyzer;
use crate::kubernetes::configmaps::ConfigMapValidator;
use crate::kubernetes::cronjobs::CronJobValidator;
use crate::kubernetes::daemonsets::DaemonSetValidator;
use crate::kubernetes::deployments::DeploymentValidator;
use crate::kubernetes::ingress::IngressValidator;
use crate::kubernetes::jobs::JobValidator;
use crate::kubernetes::namespaces::NamespaceValidator;
use crate::kubernetes::networking::NetworkPolicyValidator;
use crate::kubernetes::pvc::PvcValidator;
use crate::kubernetes::resource_detector::{K8sResource, K8sResourceDetector, K8sResourceKind};
use crate::kubernetes::secrets::SecretValidator;
use crate::kubernetes::security::K8sSecurityAnalyzer;
use crate::kubernetes::services::ServiceValidator;
use crate::kubernetes::statefulsets::StatefulSetValidator;
use crate::kubernetes::storage::StorageValidator;
use crate::yaml::parser::{Span, YamlAST};

/// Diagnostic severity for Kubernetes validation findings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum K8sSeverity {
    Error,
    Warning,
    Info,
    Hint,
}

/// Primary diagnostic item produced by `K8sValidator`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct K8sDiagnostic {
    /// Rule identifier (e.g. `k8s-invalid-apiversion`, `sec-k8s-no-privileged`, `rel-k8s-dangling-ingress`).
    pub rule_id: String,
    /// Human-readable message.
    pub message: String,
    /// Severity level.
    pub severity: K8sSeverity,
    /// Target span location in source text.
    pub span: Span,
    /// Automated quick-fix text replacement proposal.
    pub quick_fix: Option<(String, String)>,
}

/// Central Kubernetes validator coordinating manifest analysis and relationship graph checks.
#[derive(Debug, Clone, Default)]
pub struct K8sValidator {
    api_version_evaluator: K8sApiVersionEvaluator,
    security_analyzer: K8sSecurityAnalyzer,
    best_practices_analyzer: K8sBestPracticesAnalyzer,
    deployment_validator: DeploymentValidator,
    statefulset_validator: StatefulSetValidator,
    daemonset_validator: DaemonSetValidator,
    job_validator: JobValidator,
    cronjob_validator: CronJobValidator,
    service_validator: ServiceValidator,
    ingress_validator: IngressValidator,
    configmap_validator: ConfigMapValidator,
    secret_validator: SecretValidator,
    pvc_validator: PvcValidator,
    storage_validator: StorageValidator,
    namespace_validator: NamespaceValidator,
    network_policy_validator: NetworkPolicyValidator,
}

impl K8sValidator {
    /// Creates a new K8sValidator.
    pub fn new() -> Self {
        Self::default()
    }

    /// Validates a parsed `YamlAST` containing Kubernetes manifests.
    pub fn validate_ast(&self, ast: &YamlAST) -> Vec<K8sDiagnostic> {
        let mut detector = K8sResourceDetector::new();
        let resources = detector.detect_resources(ast);
        let mut diagnostics = Vec::new();

        for res in &resources {
            // 1. Check apiVersion status
            match self
                .api_version_evaluator
                .evaluate(res.kind, &res.api_version)
            {
                ApiVersionStatus::Valid => {}
                ApiVersionStatus::Deprecated { ref replacement } => {
                    diagnostics.push(K8sDiagnostic {
                        rule_id: "k8s-deprecated-apiversion".to_string(),
                        message: format!(
                            "apiVersion '{}' for {} is deprecated; use '{}'",
                            res.api_version, res.kind, replacement
                        ),
                        severity: K8sSeverity::Warning,
                        span: res.span,
                        quick_fix: Some((
                            format!("Update apiVersion to '{replacement}'"),
                            replacement.clone(),
                        )),
                    });
                }
                ApiVersionStatus::Invalid { ref expected } => {
                    diagnostics.push(K8sDiagnostic {
                        rule_id: "k8s-invalid-apiversion".to_string(),
                        message: format!(
                            "Invalid apiVersion '{}' for {}; expected {:?}",
                            res.api_version, res.kind, expected
                        ),
                        severity: K8sSeverity::Error,
                        span: res.span,
                        quick_fix: None,
                    });
                }
            }

            // 2. Resource-specific spec validation
            match res.kind {
                K8sResourceKind::Deployment => {
                    for issue in self.deployment_validator.validate(res) {
                        diagnostics.push(K8sDiagnostic {
                            rule_id: issue.rule_id,
                            message: issue.message,
                            severity: K8sSeverity::Error,
                            span: res.span,
                            quick_fix: None,
                        });
                    }
                }
                K8sResourceKind::StatefulSet => {
                    for issue in self.statefulset_validator.validate(res) {
                        diagnostics.push(K8sDiagnostic {
                            rule_id: issue.rule_id,
                            message: issue.message,
                            severity: K8sSeverity::Error,
                            span: res.span,
                            quick_fix: None,
                        });
                    }
                }
                K8sResourceKind::DaemonSet => {
                    for issue in self.daemonset_validator.validate(res) {
                        diagnostics.push(K8sDiagnostic {
                            rule_id: issue.rule_id,
                            message: issue.message,
                            severity: K8sSeverity::Error,
                            span: res.span,
                            quick_fix: None,
                        });
                    }
                }
                K8sResourceKind::Job => {
                    for issue in self.job_validator.validate(res) {
                        diagnostics.push(K8sDiagnostic {
                            rule_id: issue.rule_id,
                            message: issue.message,
                            severity: K8sSeverity::Error,
                            span: res.span,
                            quick_fix: None,
                        });
                    }
                }
                K8sResourceKind::CronJob => {
                    for issue in self.cronjob_validator.validate(res) {
                        diagnostics.push(K8sDiagnostic {
                            rule_id: issue.rule_id,
                            message: issue.message,
                            severity: K8sSeverity::Error,
                            span: res.span,
                            quick_fix: None,
                        });
                    }
                }
                K8sResourceKind::Service => {
                    for issue in self.service_validator.validate(res) {
                        diagnostics.push(K8sDiagnostic {
                            rule_id: issue.rule_id,
                            message: issue.message,
                            severity: K8sSeverity::Error,
                            span: res.span,
                            quick_fix: None,
                        });
                    }
                }
                K8sResourceKind::Ingress => {
                    for issue in self.ingress_validator.validate(res) {
                        diagnostics.push(K8sDiagnostic {
                            rule_id: issue.rule_id,
                            message: issue.message,
                            severity: K8sSeverity::Error,
                            span: res.span,
                            quick_fix: None,
                        });
                    }
                }
                K8sResourceKind::ConfigMap => {
                    for issue in self.configmap_validator.validate(res) {
                        diagnostics.push(K8sDiagnostic {
                            rule_id: issue.rule_id,
                            message: issue.message,
                            severity: K8sSeverity::Warning,
                            span: res.span,
                            quick_fix: None,
                        });
                    }
                }
                K8sResourceKind::Secret => {
                    for issue in self.secret_validator.validate(res) {
                        diagnostics.push(K8sDiagnostic {
                            rule_id: issue.rule_id,
                            message: issue.message,
                            severity: K8sSeverity::Error,
                            span: res.span,
                            quick_fix: None,
                        });
                    }
                }
                K8sResourceKind::PersistentVolumeClaim => {
                    for issue in self.pvc_validator.validate(res) {
                        diagnostics.push(K8sDiagnostic {
                            rule_id: issue.rule_id,
                            message: issue.message,
                            severity: K8sSeverity::Error,
                            span: res.span,
                            quick_fix: None,
                        });
                    }
                }
                K8sResourceKind::StorageClass | K8sResourceKind::PersistentVolume => {
                    for issue in self.storage_validator.validate(res) {
                        diagnostics.push(K8sDiagnostic {
                            rule_id: issue.rule_id,
                            message: issue.message,
                            severity: K8sSeverity::Error,
                            span: res.span,
                            quick_fix: None,
                        });
                    }
                }
                K8sResourceKind::Namespace => {
                    for issue in self.namespace_validator.validate(res) {
                        diagnostics.push(K8sDiagnostic {
                            rule_id: issue.rule_id,
                            message: issue.message,
                            severity: K8sSeverity::Error,
                            span: res.span,
                            quick_fix: None,
                        });
                    }
                }
                K8sResourceKind::NetworkPolicy => {
                    for issue in self.network_policy_validator.validate(res) {
                        diagnostics.push(K8sDiagnostic {
                            rule_id: issue.rule_id,
                            message: issue.message,
                            severity: K8sSeverity::Error,
                            span: res.span,
                            quick_fix: None,
                        });
                    }
                }
                _ => {}
            }

            // 3. Security Analysis
            let security_findings = self.security_analyzer.audit(res);
            for finding in security_findings {
                let severity = match finding.severity {
                    crate::kubernetes::security::SecuritySeverity::Critical
                    | crate::kubernetes::security::SecuritySeverity::High => K8sSeverity::Error,
                    crate::kubernetes::security::SecuritySeverity::Medium => K8sSeverity::Warning,
                    crate::kubernetes::security::SecuritySeverity::Low => K8sSeverity::Info,
                };
                let quick_fix = finding
                    .fix_suggestion
                    .map(|fix| ("Apply security fix".to_string(), fix));

                diagnostics.push(K8sDiagnostic {
                    rule_id: finding.rule_id,
                    message: finding.message,
                    severity,
                    span: finding.span,
                    quick_fix,
                });
            }

            // 4. Best Practices Recommendations
            let recommendations = self.best_practices_analyzer.evaluate(res);
            for rec in recommendations {
                diagnostics.push(K8sDiagnostic {
                    rule_id: rec.rule_id,
                    message: rec.message,
                    severity: K8sSeverity::Hint,
                    span: res.span,
                    quick_fix: None,
                });
            }
        }

        // 5. Cross-Resource Relationship Validation
        self.validate_relationships(&resources, &mut diagnostics);

        diagnostics
    }

    fn validate_relationships(
        &self,
        resources: &[K8sResource],
        diagnostics: &mut Vec<K8sDiagnostic>,
    ) {
        let services: Vec<_> = resources
            .iter()
            .filter(|r| r.kind == K8sResourceKind::Service)
            .collect();
        let ingresses: Vec<_> = resources
            .iter()
            .filter(|r| r.kind == K8sResourceKind::Ingress)
            .collect();

        // Validate Ingress -> Service relationships
        for ing in ingresses {
            let svc_refs = extract_ingress_backend_services(&ing.node, &ing.metadata.name);
            for svc_ref_name in svc_refs {
                let exists = services.iter().any(|s| {
                    s.metadata.name == svc_ref_name
                        && s.metadata.namespace == ing.metadata.namespace
                });

                if !exists {
                    diagnostics.push(K8sDiagnostic {
                        rule_id: "rel-k8s-dangling-ingress".to_string(),
                        message: format!(
                            "Ingress '{}' references Service '{}' in namespace '{}' which is not defined",
                            ing.metadata.name, svc_ref_name, ing.metadata.namespace
                        ),
                        severity: K8sSeverity::Error,
                        span: ing.span,
                        quick_fix: None,
                    });
                }
            }
        }
    }
}

fn extract_ingress_backend_services(
    root_node: &crate::yaml::parser::YamlNode,
    ingress_name: &str,
) -> Vec<String> {
    let mut names = Vec::new();
    find_backend_services_recursive(&root_node.value, false, ingress_name, &mut names);
    names
}

fn find_backend_services_recursive(
    val: &crate::yaml::parser::YamlValue,
    in_backend: bool,
    ingress_name: &str,
    names: &mut Vec<String>,
) {
    match val {
        crate::yaml::parser::YamlValue::Mapping(ref map) => {
            let mut active_backend = in_backend;
            for pair in &map.pairs {
                if pair.key.value == "backend" || pair.key.value == "service" {
                    active_backend = true;
                }
                if active_backend && (pair.key.value == "name" || pair.key.value == "serviceName") {
                    if let crate::yaml::parser::YamlValue::Scalar(ref s) = pair.value.value {
                        let v = s.value.trim();
                        if !v.is_empty() && v != ingress_name && !names.contains(&v.to_string()) {
                            names.push(v.to_string());
                        }
                    }
                }
                find_backend_services_recursive(
                    &pair.value.value,
                    active_backend,
                    ingress_name,
                    names,
                );
            }
        }
        crate::yaml::parser::YamlValue::Sequence(ref seq) => {
            for item in &seq.items {
                find_backend_services_recursive(&item.value, in_backend, ingress_name, names);
            }
        }
        _ => {}
    }
}
