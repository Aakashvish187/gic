//! Kubernetes Schema Definition and Property Specification Contracts.

use crate::kubernetes::resource_detector::K8sResourceKind;

/// Property schema descriptor for a Kubernetes field.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct K8sSchemaProperty {
    /// Field path name.
    pub name: String,
    /// Detailed description.
    pub description: String,
    /// Expected data type.
    pub data_type: &'static str,
    /// True if required field.
    pub required: bool,
    /// Official Kubernetes documentation URL.
    pub doc_link: Option<String>,
}

/// Contract for a complete Kubernetes Resource Schema.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct K8sResourceSchema {
    /// Target resource kind.
    pub kind: K8sResourceKind,
    /// Primary apiVersion.
    pub default_api_version: &'static str,
    /// Property definitions.
    pub properties: Vec<K8sSchemaProperty>,
}
