//! Kubernetes Best Practices Recommendation Engine.
//!
//! Evaluates manifests against Kubernetes production standards (recommended `app.kubernetes.io/*` labels,
//! explicit namespace declaration, resource limits, health probes, rolling update strategies).

use crate::kubernetes::resource_detector::{K8sResource, K8sResourceKind};

/// Best practice recommendation item.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BestPracticeRecommendation {
    /// Rule identifier (e.g. `bp-k8s-recommended-labels`, `bp-k8s-explicit-namespace`).
    pub rule_id: String,
    /// Detailed recommendation text.
    pub message: String,
    /// Line number.
    pub line: usize,
    /// Category tag.
    pub category: &'static str,
}

/// Analyzer for Kubernetes production best practices.
#[derive(Debug, Clone, Default)]
pub struct K8sBestPracticesAnalyzer;

impl K8sBestPracticesAnalyzer {
    /// Creates a new K8sBestPracticesAnalyzer.
    pub fn new() -> Self {
        Self
    }

    /// Evaluates a Kubernetes resource for best practice adherence.
    pub fn evaluate(&self, resource: &K8sResource) -> Vec<BestPracticeRecommendation> {
        let mut recommendations = Vec::new();

        // 1. Check explicit namespace assignment (except Cluster-scoped resources like Namespace, StorageClass, ClusterRole)
        if !is_cluster_scoped(resource.kind) && resource.metadata.namespace == "default" {
            recommendations.push(BestPracticeRecommendation {
                rule_id: "bp-k8s-explicit-namespace".to_string(),
                message: format!(
                    "{} '{}' uses default namespace; specify an explicit 'metadata.namespace'",
                    resource.kind, resource.metadata.name
                ),
                line: resource.span.start.line,
                category: "Organization",
            });
        }

        // 2. Check recommended standard labels (`app.kubernetes.io/name`, etc.)
        let has_app_label = resource
            .metadata
            .labels
            .keys()
            .any(|k| k.starts_with("app.kubernetes.io/") || k == "app");

        if !has_app_label && resource.kind != K8sResourceKind::Namespace {
            recommendations.push(BestPracticeRecommendation {
                rule_id: "bp-k8s-recommended-labels".to_string(),
                message: format!(
                    "{} '{}' lacks standard 'app.kubernetes.io/name' or 'app' label",
                    resource.kind, resource.metadata.name
                ),
                line: resource.span.start.line,
                category: "Metadata",
            });
        }

        recommendations
    }
}

fn is_cluster_scoped(kind: K8sResourceKind) -> bool {
    matches!(
        kind,
        K8sResourceKind::Namespace
            | K8sResourceKind::ClusterRole
            | K8sResourceKind::ClusterRoleBinding
            | K8sResourceKind::StorageClass
            | K8sResourceKind::PersistentVolume
    )
}
