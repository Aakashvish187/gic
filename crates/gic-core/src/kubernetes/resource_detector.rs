//! Kubernetes Resource Detection and Metadata Extraction Engine.
//!
//! Scans a `YamlAST` to identify Kubernetes manifest documents, resolve resource `kind`
//! and `apiVersion`, and extract `metadata` (name, namespace, labels, annotations).

use std::collections::HashMap;
use std::fmt;

use crate::yaml::parser::{Span, YamlAST, YamlMapping, YamlNode, YamlValue};

/// Supported Kubernetes resource kind classifications.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum K8sResourceKind {
    Deployment,
    StatefulSet,
    DaemonSet,
    ReplicaSet,
    Pod,
    Service,
    Ingress,
    Namespace,
    ConfigMap,
    Secret,
    PersistentVolume,
    PersistentVolumeClaim,
    StorageClass,
    Job,
    CronJob,
    ServiceAccount,
    Role,
    RoleBinding,
    ClusterRole,
    ClusterRoleBinding,
    NetworkPolicy,
    CustomResource,
}

impl K8sResourceKind {
    /// Attempts to parse a string slice into a `K8sResourceKind`.
    pub fn from_str(kind: &str) -> Option<Self> {
        match kind {
            "Deployment" => Some(Self::Deployment),
            "StatefulSet" => Some(Self::StatefulSet),
            "DaemonSet" => Some(Self::DaemonSet),
            "ReplicaSet" => Some(Self::ReplicaSet),
            "Pod" => Some(Self::Pod),
            "Service" => Some(Self::Service),
            "Ingress" => Some(Self::Ingress),
            "Namespace" => Some(Self::Namespace),
            "ConfigMap" => Some(Self::ConfigMap),
            "Secret" => Some(Self::Secret),
            "PersistentVolume" => Some(Self::PersistentVolume),
            "PersistentVolumeClaim" => Some(Self::PersistentVolumeClaim),
            "StorageClass" => Some(Self::StorageClass),
            "Job" => Some(Self::Job),
            "CronJob" => Some(Self::CronJob),
            "ServiceAccount" => Some(Self::ServiceAccount),
            "Role" => Some(Self::Role),
            "RoleBinding" => Some(Self::RoleBinding),
            "ClusterRole" => Some(Self::ClusterRole),
            "ClusterRoleBinding" => Some(Self::ClusterRoleBinding),
            "NetworkPolicy" => Some(Self::NetworkPolicy),
            _ => None,
        }
    }

    /// Returns standard Kubernetes Kind string label.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Deployment => "Deployment",
            Self::StatefulSet => "StatefulSet",
            Self::DaemonSet => "DaemonSet",
            Self::ReplicaSet => "ReplicaSet",
            Self::Pod => "Pod",
            Self::Service => "Service",
            Self::Ingress => "Ingress",
            Self::Namespace => "Namespace",
            Self::ConfigMap => "ConfigMap",
            Self::Secret => "Secret",
            Self::PersistentVolume => "PersistentVolume",
            Self::PersistentVolumeClaim => "PersistentVolumeClaim",
            Self::StorageClass => "StorageClass",
            Self::Job => "Job",
            Self::CronJob => "CronJob",
            Self::ServiceAccount => "ServiceAccount",
            Self::Role => "Role",
            Self::RoleBinding => "RoleBinding",
            Self::ClusterRole => "ClusterRole",
            Self::ClusterRoleBinding => "ClusterRoleBinding",
            Self::NetworkPolicy => "NetworkPolicy",
            Self::CustomResource => "CustomResource",
        }
    }
}

impl fmt::Display for K8sResourceKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Metadata section extracted from a Kubernetes manifest (`metadata:` block).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ResourceMetadata {
    /// Object name (`metadata.name`).
    pub name: String,
    /// Object namespace (`metadata.namespace`), defaults to `"default"`.
    pub namespace: String,
    /// Labels dictionary (`metadata.labels`).
    pub labels: HashMap<String, String>,
    /// Annotations dictionary (`metadata.annotations`).
    pub annotations: HashMap<String, String>,
    /// Span of the metadata block.
    pub span: Span,
}

/// Parsed Kubernetes resource representation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct K8sResource {
    /// Unique identifier for indexing in graph engine.
    pub id: usize,
    /// Evaluated Kind.
    pub kind: K8sResourceKind,
    /// Raw kind string from manifest.
    pub raw_kind: String,
    /// API Version string.
    pub api_version: String,
    /// Extracted metadata.
    pub metadata: ResourceMetadata,
    /// Span covering entire resource manifest document.
    pub span: Span,
    /// Reference to underlying YAML AST root node.
    pub node: YamlNode,
}

/// Detector for recognizing and extracting Kubernetes resources from a YAML AST.
#[derive(Debug, Clone, Default)]
pub struct K8sResourceDetector {
    next_id: usize,
}

impl K8sResourceDetector {
    /// Creates a new K8sResourceDetector.
    pub fn new() -> Self {
        Self { next_id: 1 }
    }

    fn allocate_id(&mut self) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    /// Detects all Kubernetes manifest resources present in a `YamlAST`.
    pub fn detect_resources(&mut self, ast: &YamlAST) -> Vec<K8sResource> {
        let mut resources = Vec::new();

        for doc in &ast.documents {
            if let Some(ref root) = doc.root {
                if let YamlValue::Mapping(ref map) = root.value {
                    if let Some(res) = self.extract_resource_from_mapping(root, map, doc.span) {
                        resources.push(res);
                    }
                }
            }
        }

        resources
    }

    fn extract_resource_from_mapping(
        &mut self,
        root_node: &YamlNode,
        map: &YamlMapping,
        span: Span,
    ) -> Option<K8sResource> {
        let mut raw_kind = None;
        let mut api_version = None;
        let mut metadata = ResourceMetadata::default();

        for pair in &map.pairs {
            match pair.key.value.as_str() {
                "kind" => {
                    if let YamlValue::Scalar(ref s) = pair.value.value {
                        raw_kind = Some(s.value.clone());
                    }
                }
                "apiVersion" => {
                    if let YamlValue::Scalar(ref s) = pair.value.value {
                        api_version = Some(s.value.clone());
                    }
                }
                "name" if metadata.name.is_empty() => {
                    if let YamlValue::Scalar(ref s) = pair.value.value {
                        metadata.name = s.value.clone();
                    }
                }
                "namespace" if metadata.namespace.is_empty() => {
                    if let YamlValue::Scalar(ref s) = pair.value.value {
                        metadata.namespace = s.value.clone();
                    }
                }
                _ => {}
            }
        }

        let kind_str = raw_kind?;
        let api_ver_str = api_version?;

        let kind = K8sResourceKind::from_str(&kind_str).unwrap_or(K8sResourceKind::CustomResource);
        let id = self.allocate_id();

        if metadata.namespace.is_empty() {
            metadata.namespace = "default".to_string();
        }

        Some(K8sResource {
            id,
            kind,
            raw_kind: kind_str,
            api_version: api_ver_str,
            metadata,
            span,
            node: root_node.clone(),
        })
    }
}
