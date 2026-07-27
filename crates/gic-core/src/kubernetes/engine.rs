//! Central Kubernetes Intelligence Engine.
//!
//! Unified façade coordinating resource detection, manifest validation, security auditing,
//! production best practice recommendations, cross-resource relationship graph validation,
//! completion, hover documentation, and incremental result caching.

use std::sync::Arc;

use crate::diagnostics::diagnostic::Diagnostic;
use crate::kubernetes::cache::K8sCache;
use crate::kubernetes::completion::{K8sCompletionContext, K8sCompletionEngine, K8sCompletionItem};
use crate::kubernetes::diagnostics::convert_k8s_diagnostics;
use crate::kubernetes::errors::K8sResult;
use crate::kubernetes::hover::{K8sHoverContext, K8sHoverEngine, K8sHoverInfo};
use crate::kubernetes::resource_detector::{K8sResource, K8sResourceDetector};
use crate::kubernetes::validator::{K8sDiagnostic, K8sValidator};
use crate::yaml::parser::YamlParser;

/// Options controlling `K8sEngine` execution.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct K8sEngineOptions {
    /// Maximum capacity of incremental cache entries.
    pub cache_capacity: usize,
}

/// Primary Kubernetes Intelligence Engine façade.
pub struct K8sEngine {
    validator: K8sValidator,
    completion_engine: K8sCompletionEngine,
    hover_engine: K8sHoverEngine,
    cache: Arc<K8sCache>,
}

impl Default for K8sEngine {
    fn default() -> Self {
        Self::new(K8sEngineOptions {
            cache_capacity: 100,
        })
    }
}

impl K8sEngine {
    /// Creates a new K8sEngine.
    pub fn new(options: K8sEngineOptions) -> Self {
        Self {
            validator: K8sValidator::new(),
            completion_engine: K8sCompletionEngine::new(),
            hover_engine: K8sHoverEngine::new(),
            cache: Arc::new(K8sCache::new(options.cache_capacity)),
        }
    }

    /// Detects all Kubernetes resources in raw YAML source.
    pub fn detect_resources(&self, source: &str) -> K8sResult<Vec<K8sResource>> {
        let mut yaml_parser = YamlParser::new();
        let ast = yaml_parser
            .parse(source)
            .map_err(|e| crate::kubernetes::errors::K8sError::YamlError(e.to_string()))?;
        let mut detector = K8sResourceDetector::new();
        Ok(detector.detect_resources(&ast))
    }

    /// Validates raw Kubernetes YAML source code and returns both internal and central `Diagnostic` items.
    pub fn validate(&self, source: &str) -> (Vec<K8sDiagnostic>, Vec<Diagnostic>) {
        if let Some(entry) = self.cache.get(source) {
            let core_diags = convert_k8s_diagnostics(entry.diagnostics.clone());
            return (entry.diagnostics, core_diags);
        }

        let mut yaml_parser = YamlParser::new();
        let mut internal_diags = Vec::new();

        if let Ok(ast) = yaml_parser.parse(source) {
            let k8s_diags = self.validator.validate_ast(&ast);
            internal_diags.extend(k8s_diags);

            let mut detector = K8sResourceDetector::new();
            let resources = detector.detect_resources(&ast);
            self.cache.put(source, resources, internal_diags.clone());
        }

        let core_diags = convert_k8s_diagnostics(internal_diags.clone());
        (internal_diags, core_diags)
    }

    /// Queries autocomplete recommendations for position in manifest source.
    pub fn autocomplete(&self, ctx: &K8sCompletionContext) -> Vec<K8sCompletionItem> {
        self.completion_engine.complete(ctx)
    }

    /// Queries hover documentation tooltip for position in manifest source.
    pub fn hover(&self, ctx: &K8sHoverContext) -> Option<K8sHoverInfo> {
        self.hover_engine.hover(ctx)
    }

    /// Reference to the cache instance.
    pub fn cache(&self) -> &K8sCache {
        &self.cache
    }
}
