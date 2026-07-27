//! Kubernetes apiVersion Validator and Version Compatibility Checker.
//!
//! Validates `apiVersion` strings against Kubernetes resource kinds and flags deprecated
//! or obsolete API group versions.

use crate::kubernetes::resource_detector::K8sResourceKind;

/// Status of an apiVersion for a specific resource kind.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApiVersionStatus {
    /// Valid and current API version.
    Valid,
    /// Deprecated API version with replacement version.
    Deprecated { replacement: String },
    /// Completely invalid or unsupported API version for kind.
    Invalid { expected: Vec<String> },
}

/// Evaluates apiVersion strings against Kubernetes resource kinds.
#[derive(Debug, Clone, Default)]
pub struct K8sApiVersionEvaluator;

impl K8sApiVersionEvaluator {
    /// Creates a new K8sApiVersionEvaluator.
    pub fn new() -> Self {
        Self
    }

    /// Evaluates `api_version` for a given `K8sResourceKind`.
    pub fn evaluate(&self, kind: K8sResourceKind, api_version: &str) -> ApiVersionStatus {
        match kind {
            K8sResourceKind::Deployment
            | K8sResourceKind::StatefulSet
            | K8sResourceKind::DaemonSet
            | K8sResourceKind::ReplicaSet => {
                if api_version == "apps/v1" {
                    ApiVersionStatus::Valid
                } else if api_version == "apps/v1beta1"
                    || api_version == "apps/v1beta2"
                    || api_version == "extensions/v1beta1"
                {
                    ApiVersionStatus::Deprecated {
                        replacement: "apps/v1".to_string(),
                    }
                } else {
                    ApiVersionStatus::Invalid {
                        expected: vec!["apps/v1".to_string()],
                    }
                }
            }
            K8sResourceKind::Pod
            | K8sResourceKind::Service
            | K8sResourceKind::Namespace
            | K8sResourceKind::ConfigMap
            | K8sResourceKind::Secret
            | K8sResourceKind::PersistentVolume
            | K8sResourceKind::PersistentVolumeClaim
            | K8sResourceKind::ServiceAccount => {
                if api_version == "v1" {
                    ApiVersionStatus::Valid
                } else {
                    ApiVersionStatus::Invalid {
                        expected: vec!["v1".to_string()],
                    }
                }
            }
            K8sResourceKind::Ingress | K8sResourceKind::NetworkPolicy => {
                if api_version == "networking.k8s.io/v1" {
                    ApiVersionStatus::Valid
                } else if api_version == "extensions/v1beta1"
                    || api_version == "networking.k8s.io/v1beta1"
                {
                    ApiVersionStatus::Deprecated {
                        replacement: "networking.k8s.io/v1".to_string(),
                    }
                } else {
                    ApiVersionStatus::Invalid {
                        expected: vec!["networking.k8s.io/v1".to_string()],
                    }
                }
            }
            K8sResourceKind::Job => {
                if api_version == "batch/v1" {
                    ApiVersionStatus::Valid
                } else {
                    ApiVersionStatus::Invalid {
                        expected: vec!["batch/v1".to_string()],
                    }
                }
            }
            K8sResourceKind::CronJob => {
                if api_version == "batch/v1" {
                    ApiVersionStatus::Valid
                } else if api_version == "batch/v1beta1" {
                    ApiVersionStatus::Deprecated {
                        replacement: "batch/v1".to_string(),
                    }
                } else {
                    ApiVersionStatus::Invalid {
                        expected: vec!["batch/v1".to_string()],
                    }
                }
            }
            K8sResourceKind::Role
            | K8sResourceKind::RoleBinding
            | K8sResourceKind::ClusterRole
            | K8sResourceKind::ClusterRoleBinding => {
                if api_version == "rbac.authorization.k8s.io/v1" {
                    ApiVersionStatus::Valid
                } else if api_version == "rbac.authorization.k8s.io/v1beta1" {
                    ApiVersionStatus::Deprecated {
                        replacement: "rbac.authorization.k8s.io/v1".to_string(),
                    }
                } else {
                    ApiVersionStatus::Invalid {
                        expected: vec!["rbac.authorization.k8s.io/v1".to_string()],
                    }
                }
            }
            K8sResourceKind::StorageClass => {
                if api_version == "storage.k8s.io/v1" {
                    ApiVersionStatus::Valid
                } else {
                    ApiVersionStatus::Invalid {
                        expected: vec!["storage.k8s.io/v1".to_string()],
                    }
                }
            }
            K8sResourceKind::CustomResource => ApiVersionStatus::Valid,
        }
    }
}
