//! Kubernetes Autocomplete Architecture and Interfaces.

use crate::kubernetes::resource_detector::K8sResourceKind;
use crate::yaml::parser::Position;

/// Category of Kubernetes completion suggestion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum K8sCompletionKind {
    Kind,
    ApiVersion,
    Field,
    Value,
    Snippet,
}

/// Suggested completion item for Kubernetes manifests.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct K8sCompletionItem {
    pub label: String,
    pub insert_text: String,
    pub detail: Option<String>,
    pub documentation: Option<String>,
    pub kind: K8sCompletionKind,
}

/// Context for Kubernetes completion evaluation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct K8sCompletionContext {
    pub position: Position,
    pub resource_kind: Option<K8sResourceKind>,
    pub key_path: Vec<String>,
}

/// Trait interface for future Kubernetes completion providers.
pub trait K8sCompletionProvider: Send + Sync {
    fn name(&self) -> &str;
    fn complete(&self, ctx: &K8sCompletionContext) -> Vec<K8sCompletionItem>;
}

/// Kubernetes completion engine coordinator.
#[derive(Default)]
pub struct K8sCompletionEngine {
    providers: Vec<Box<dyn K8sCompletionProvider>>,
}

impl K8sCompletionEngine {
    pub fn new() -> Self {
        Self {
            providers: Vec::new(),
        }
    }

    pub fn register_provider(&mut self, provider: Box<dyn K8sCompletionProvider>) {
        self.providers.push(provider);
    }

    pub fn complete(&self, ctx: &K8sCompletionContext) -> Vec<K8sCompletionItem> {
        let mut items = Vec::new();
        for p in &self.providers {
            items.extend(p.complete(ctx));
        }
        items
    }
}
