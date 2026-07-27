//! Kubernetes Hover Documentation Architecture and Interfaces.

use crate::kubernetes::resource_detector::K8sResourceKind;
use crate::yaml::parser::{Position, Span};

/// Hover tooltip info for Kubernetes fields and resources.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct K8sHoverInfo {
    pub markdown: String,
    pub span: Span,
    pub doc_link: Option<String>,
}

/// Context for Kubernetes hover evaluation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct K8sHoverContext {
    pub position: Position,
    pub resource_kind: Option<K8sResourceKind>,
    pub key_path: Vec<String>,
}

/// Trait interface for future Kubernetes hover providers.
pub trait K8sHoverProvider: Send + Sync {
    fn name(&self) -> &str;
    fn hover(&self, ctx: &K8sHoverContext) -> Option<K8sHoverInfo>;
}

/// Kubernetes hover engine coordinator.
#[derive(Default)]
pub struct K8sHoverEngine {
    providers: Vec<Box<dyn K8sHoverProvider>>,
}

impl K8sHoverEngine {
    pub fn new() -> Self {
        Self {
            providers: Vec::new(),
        }
    }

    pub fn register_provider(&mut self, provider: Box<dyn K8sHoverProvider>) {
        self.providers.push(provider);
    }

    pub fn hover(&self, ctx: &K8sHoverContext) -> Option<K8sHoverInfo> {
        for p in &self.providers {
            if let Some(info) = p.hover(ctx) {
                return Some(info);
            }
        }
        None
    }
}
