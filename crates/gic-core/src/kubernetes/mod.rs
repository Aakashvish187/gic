//! Kubernetes Intelligence Engine module for GIC (General Infrastructure Console).
//!
//! Provides production-grade Kubernetes manifest analysis, resource detection for 21+ Kubernetes
//! resource kinds, security auditing (PodSecurityStandards), production best practices recommendations,
//! cross-resource relationship graph validation, completion/hover contracts, and incremental caching.

#![forbid(unsafe_code)]

pub mod api_version;
pub mod best_practices;
pub mod cache;
pub mod completion;
pub mod configmaps;
pub mod cronjobs;
pub mod daemonsets;
pub mod deployments;
pub mod diagnostics;
pub mod engine;
pub mod errors;
pub mod hover;
pub mod ingress;
pub mod jobs;
pub mod labels;
pub mod namespaces;
pub mod networking;
pub mod probes;
pub mod pvc;
pub mod resource_detector;
pub mod resources;
pub mod schema;
pub mod secrets;
pub mod security;
pub mod selectors;
pub mod services;
pub mod statefulsets;
pub mod storage;
pub mod validator;

#[cfg(test)]
pub mod tests;

pub use api_version::{ApiVersionStatus, K8sApiVersionEvaluator};
pub use best_practices::{BestPracticeRecommendation, K8sBestPracticesAnalyzer};
pub use cache::{K8sCache, K8sCacheEntry, K8sCacheMetrics};
pub use completion::{
    K8sCompletionContext, K8sCompletionEngine, K8sCompletionItem, K8sCompletionKind,
    K8sCompletionProvider,
};
pub use configmaps::{ConfigMapIssue, ConfigMapValidator};
pub use cronjobs::{CronJobIssue, CronJobValidator};
pub use daemonsets::{DaemonSetIssue, DaemonSetValidator};
pub use deployments::{DeploymentIssue, DeploymentValidator};
pub use diagnostics::{convert_k8s_diagnostic, convert_k8s_diagnostics};
pub use engine::{K8sEngine, K8sEngineOptions};
pub use errors::{K8sError, K8sResult};
pub use hover::{K8sHoverContext, K8sHoverEngine, K8sHoverInfo, K8sHoverProvider};
pub use ingress::{IngressIssue, IngressValidator};
pub use jobs::{JobIssue, JobValidator};
pub use labels::{LabelIssue, LabelMap, LabelValidator};
pub use namespaces::{NamespaceIssue, NamespaceValidator};
pub use networking::{NetworkPolicyIssue, NetworkPolicyValidator};
pub use probes::{ContainerProbeReport, K8sProbeAnalyzer, ProbeConfig, ProbeType};
pub use pvc::{PvcIssue, PvcValidator};
pub use resource_detector::{K8sResource, K8sResourceDetector, K8sResourceKind, ResourceMetadata};
pub use resources::{ContainerResourceReport, K8sResourceRequirementsAnalyzer, ResourceSpec};
pub use schema::{K8sResourceSchema, K8sSchemaProperty};
pub use secrets::{SecretIssue, SecretValidator};
pub use security::{K8sSecurityAnalyzer, SecurityFinding, SecuritySeverity};
pub use selectors::{LabelSelector, SelectorOperator, SelectorRequirement};
pub use services::{ServiceIssue, ServiceValidator};
pub use statefulsets::{StatefulSetIssue, StatefulSetValidator};
pub use storage::{StorageIssue, StorageValidator};
pub use validator::{K8sDiagnostic, K8sSeverity, K8sValidator};
