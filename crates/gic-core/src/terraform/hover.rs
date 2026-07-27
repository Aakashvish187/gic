//! Hover Documentation Interface Contracts for Terraform Engine.
//!
//! Prepares contracts and interfaces for hovering over providers, resources, attributes,
//! and modules to display documentation, Terraform Registry URLs, deprecation notices, and examples.

use crate::yaml::parser::Position;

/// Hover documentation payload item.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct HoverDoc {
    /// Markdown documentation content.
    pub markdown_content: String,
    /// Optional Terraform Registry online documentation link.
    pub registry_url: Option<String>,
    /// Optional deprecation notice warning text.
    pub deprecation_notice: Option<String>,
}

/// Hover documentation provider trait interface.
pub trait TerraformHoverProvider: Send + Sync {
    /// Retrieves hover documentation for symbol at target position.
    fn hover(&self, source: &str, position: Position) -> Option<HoverDoc>;
}
