//! Autocomplete Interface Contracts for Terraform Engine.
//!
//! Prepares contracts and interfaces for context-aware provider, resource, argument, variable,
//! and module completion extensions (OpenTofu, Terraform Registry, LSPs).

use crate::yaml::parser::Position;

/// Autocomplete item kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CompletionKind {
    Provider,
    Resource,
    Argument,
    Variable,
    Module,
    #[default]
    Reference,
}

/// Completion suggestion item.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TerraformCompletionItem {
    /// Insert text label.
    pub label: String,
    /// Detailed detail description.
    pub detail: String,
    /// Completion kind classification.
    pub kind: CompletionKind,
    /// Optional documentation description snippet.
    pub documentation: Option<String>,
}

/// Provider completion engine trait interface.
pub trait TerraformCompleter: Send + Sync {
    /// Returns completion proposals at the target cursor position.
    fn complete(&self, source: &str, position: Position) -> Vec<TerraformCompletionItem>;
}
