//! Terraform Backend and State Locking Validator.
//!
//! Validates `backend` configurations (`s3`, `azurerm`, `gcs`, `remote`, `cloud`, `consul`, `http`, `local`, `pg`),
//! verifies encryption settings (`encrypt = true`), and state locking provisions (`dynamodb_table`).

use std::collections::HashMap;

use crate::terraform::parser::HclBlock;
use crate::yaml::parser::Span;

/// Known Terraform backend type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BackendType {
    S3,
    Azurerm,
    GCS,
    Remote,
    Cloud,
    Consul,
    Http,
    Local,
    Pg,
    #[default]
    Unknown,
}

impl BackendType {
    /// Resolves string identifier to `BackendType`.
    pub fn from_name(name: &str) -> Self {
        match name.to_lowercase().as_str() {
            "s3" => BackendType::S3,
            "azurerm" => BackendType::Azurerm,
            "gcs" => BackendType::GCS,
            "remote" => BackendType::Remote,
            "cloud" => BackendType::Cloud,
            "consul" => BackendType::Consul,
            "http" => BackendType::Http,
            "local" => BackendType::Local,
            "pg" | "postgres" => BackendType::Pg,
            _ => BackendType::Unknown,
        }
    }
}

/// Extracted `backend` configuration node.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BackendConfiguration {
    /// Backend provider type name (e.g. `"s3"`).
    pub backend_type: String,
    /// Known backend type variant.
    pub kind: BackendType,
    /// Encrypted state flag (`encrypt = true`).
    pub is_encrypted: bool,
    /// State locking configured (e.g. `dynamodb_table` for S3).
    pub has_state_locking: bool,
    /// Backend configuration attributes.
    pub attributes: HashMap<String, String>,
    /// Source span location.
    pub span: Span,
}

/// Backend configuration validator.
#[derive(Debug, Clone, Default)]
pub struct BackendValidator;

impl BackendValidator {
    /// Creates a new BackendValidator.
    pub fn new() -> Self {
        Self
    }

    /// Extracts `BackendConfiguration` from an HCL `backend` block inside `terraform {}`.
    pub fn extract_backend(&self, block: &HclBlock) -> Option<BackendConfiguration> {
        if block.block_type != "backend" {
            return None;
        }

        let backend_type = block.first_label()?.to_string();
        let kind = BackendType::from_name(&backend_type);

        let mut is_encrypted = false;
        let mut has_state_locking = false;
        let mut attributes = HashMap::new();

        for attr in &block.attributes {
            attributes.insert(attr.name.clone(), attr.value_expression.clone());
            match attr.name.as_str() {
                "encrypt" => is_encrypted = attr.value_expression.eq_ignore_ascii_case("true"),
                "dynamodb_table" | "lock_table" | "use_azuread_auth" => has_state_locking = true,
                _ => {}
            }
        }

        if matches!(
            kind,
            BackendType::Remote | BackendType::Cloud | BackendType::GCS | BackendType::Azurerm
        ) {
            has_state_locking = true;
            is_encrypted = true;
        }

        Some(BackendConfiguration {
            backend_type,
            kind,
            is_encrypted,
            has_state_locking,
            attributes,
            span: block.span,
        })
    }
}
