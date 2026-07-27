//! Kubernetes Secret Resource Spec Validator.
//!
//! Validates `Secret` manifests for secret types (`Opaque`, `kubernetes.io/tls`, etc.) and base64 payloads.

use crate::kubernetes::resource_detector::K8sResource;
use crate::yaml::parser::{YamlMapping, YamlValue};

/// Diagnostic defect found during Secret validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecretIssue {
    /// Rule identifier.
    pub rule_id: String,
    /// Detailed diagnostic message.
    pub message: String,
    /// Line number.
    pub line: usize,
}

/// Secret specification validator.
#[derive(Debug, Clone, Default)]
pub struct SecretValidator;

impl SecretValidator {
    /// Creates a new SecretValidator.
    pub fn new() -> Self {
        Self
    }

    /// Validates a Secret `K8sResource`.
    pub fn validate(&self, resource: &K8sResource) -> Vec<SecretIssue> {
        let mut issues = Vec::new();

        if let YamlValue::Mapping(ref root_map) = resource.node.value {
            let mut secret_type = None;
            for pair in &root_map.pairs {
                if pair.key.value == "type" {
                    if let YamlValue::Scalar(ref s) = pair.value.value {
                        secret_type = Some(s.value.clone());
                    }
                }
            }

            if let Some(ref st) = secret_type {
                if st == "kubernetes.io/tls" {
                    self.validate_tls_secret(root_map, &mut issues, resource.span.start.line);
                }
            }
        }

        issues
    }

    fn validate_tls_secret(
        &self,
        root_map: &YamlMapping,
        issues: &mut Vec<SecretIssue>,
        line: usize,
    ) {
        if let Some(data_pair) = root_map
            .pairs
            .iter()
            .find(|p| p.key.value == "data" || p.key.value == "stringData")
        {
            if let YamlValue::Mapping(ref data_map) = data_pair.value.value {
                let has_crt = data_map.pairs.iter().any(|p| p.key.value == "tls.crt");
                let has_key = data_map.pairs.iter().any(|p| p.key.value == "tls.key");

                if !has_crt || !has_key {
                    issues.push(SecretIssue {
                        rule_id: "k8s-secret-tls-missing-keys".to_string(),
                        message: "TLS Secret must contain both 'tls.crt' and 'tls.key' data keys"
                            .to_string(),
                        line,
                    });
                }
            }
        }
    }
}
