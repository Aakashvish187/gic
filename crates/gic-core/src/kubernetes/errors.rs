//! Error types for the Kubernetes Intelligence Engine.

use thiserror::Error;

/// Result type alias for Kubernetes engine operations.
pub type K8sResult<T> = Result<T, K8sError>;

/// Primary error type for Kubernetes manifest validation, parsing, and relationship graph operations.
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum K8sError {
    /// Failure parsing or evaluating Kubernetes manifest.
    #[error("Kubernetes Manifest Error at L{line}:C{column}: {message}")]
    InvalidManifest {
        message: String,
        line: usize,
        column: usize,
    },

    /// Unrecognized or unsupported Kubernetes resource Kind.
    #[error("Unknown Kubernetes Kind '{kind}' at L{line}:C{column}")]
    UnknownKind {
        kind: String,
        line: usize,
        column: usize,
    },

    /// Mismatched or invalid apiVersion for given Kind.
    #[error("Invalid apiVersion '{api_version}' for Kind '{kind}' at L{line}")]
    InvalidApiVersion {
        api_version: String,
        kind: String,
        line: usize,
    },

    /// Required Kubernetes manifest field is missing.
    #[error("Missing required field '{field}' in {resource_kind} at L{line}")]
    MissingField {
        field: String,
        resource_kind: String,
        line: usize,
    },

    /// Relationship validation failure between Kubernetes resources.
    #[error("Relationship error between {source_resource} and {target_resource}: {message}")]
    RelationshipError {
        message: String,
        source_resource: String,
        target_resource: String,
    },

    /// Security policy violation (PodSecurityStandards).
    #[error("Security violation [{rule_id}] at L{line}: {message}")]
    SecurityViolation {
        message: String,
        rule_id: String,
        line: usize,
    },

    /// Cache storage or retrieval error.
    #[error("Kubernetes Cache Error: {message}")]
    CacheError { message: String },

    /// Underlying YAML error wrapping.
    #[error("YAML Engine Error: {0}")]
    YamlError(String),

    /// Generic IO error representation.
    #[error("IO Error: {0}")]
    IoError(String),
}
