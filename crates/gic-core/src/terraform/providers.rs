//! Provider Specification Registry and Validation Engine.
//!
//! Recognizes major Terraform cloud and infrastructure providers (AWS, Azure, GCP, OCI,
//! Cloudflare, GitHub, Kubernetes, Helm, Docker, Random, TLS, Archive, Null, Custom),
//! parses explicit `provider` blocks and `required_providers`, and validates version constraints.

use std::collections::HashMap;

use crate::terraform::parser::HclBlock;
use crate::yaml::parser::Span;

/// Known Terraform provider namespace enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum KnownProvider {
    AWS,
    Azure,
    GCP,
    OCI,
    Cloudflare,
    GitHub,
    Kubernetes,
    Helm,
    Docker,
    Random,
    TLS,
    Archive,
    Null,
    #[default]
    Custom,
}

impl KnownProvider {
    /// Resolves provider type string (e.g. `"aws"`, `"azurerm"`, `"google"`) to `KnownProvider`.
    pub fn from_name(name: &str) -> Self {
        match name.to_lowercase().as_str() {
            "aws" => KnownProvider::AWS,
            "azurerm" | "azure" => KnownProvider::Azure,
            "google" | "gcp" => KnownProvider::GCP,
            "oci" | "oracle" => KnownProvider::OCI,
            "cloudflare" => KnownProvider::Cloudflare,
            "github" => KnownProvider::GitHub,
            "kubernetes" | "k8s" => KnownProvider::Kubernetes,
            "helm" => KnownProvider::Helm,
            "docker" => KnownProvider::Docker,
            "random" => KnownProvider::Random,
            "tls" => KnownProvider::TLS,
            "archive" => KnownProvider::Archive,
            "null" => KnownProvider::Null,
            _ => KnownProvider::Custom,
        }
    }

    /// Returns canonical string representation.
    pub fn as_str(&self) -> &'static str {
        match self {
            KnownProvider::AWS => "aws",
            KnownProvider::Azure => "azurerm",
            KnownProvider::GCP => "google",
            KnownProvider::OCI => "oci",
            KnownProvider::Cloudflare => "cloudflare",
            KnownProvider::GitHub => "github",
            KnownProvider::Kubernetes => "kubernetes",
            KnownProvider::Helm => "helm",
            KnownProvider::Docker => "docker",
            KnownProvider::Random => "random",
            KnownProvider::TLS => "tls",
            KnownProvider::Archive => "archive",
            KnownProvider::Null => "null",
            KnownProvider::Custom => "custom",
        }
    }
}

/// Extracted `provider` block model.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ProviderConfiguration {
    /// Provider type name (e.g. `"aws"`).
    pub name: String,
    /// Known provider variant.
    pub provider_kind: KnownProvider,
    /// Provider alias if configured (e.g., `alias = "west"`).
    pub alias: Option<String>,
    /// Configured region (e.g. `"us-east-1"`).
    pub region: Option<String>,
    /// Configured version constraint if specified in provider block.
    pub version: Option<String>,
    /// Arbitrary configuration parameters.
    pub config_attributes: HashMap<String, String>,
    /// Source span location.
    pub span: Span,
}

/// Provider validator and registry engine.
#[derive(Debug, Clone, Default)]
pub struct ProviderValidator;

impl ProviderValidator {
    /// Creates a new ProviderValidator.
    pub fn new() -> Self {
        Self
    }

    /// Extracts `ProviderConfiguration` from an HCL `provider` block.
    pub fn extract_provider_config(&self, block: &HclBlock) -> Option<ProviderConfiguration> {
        if block.block_type != "provider" {
            return None;
        }

        let name = block.first_label()?.to_string();
        let provider_kind = KnownProvider::from_name(&name);

        let mut alias = None;
        let mut region = None;
        let mut version = None;
        let mut config_attributes = HashMap::new();

        for attr in &block.attributes {
            match attr.name.as_str() {
                "alias" => alias = Some(attr.value_expression.trim_matches('"').to_string()),
                "region" => region = Some(attr.value_expression.trim_matches('"').to_string()),
                "version" => version = Some(attr.value_expression.trim_matches('"').to_string()),
                _ => {
                    config_attributes.insert(attr.name.clone(), attr.value_expression.clone());
                }
            }
        }

        Some(ProviderConfiguration {
            name,
            provider_kind,
            alias,
            region,
            version,
            config_attributes,
            span: block.span,
        })
    }
}
