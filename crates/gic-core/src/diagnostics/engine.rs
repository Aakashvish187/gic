//! Core Validation Engine orchestrating validators, rules, and caching.

use crate::diagnostics::cache::DiagnosticCache;
use crate::diagnostics::diagnostic::Diagnostic;
use crate::diagnostics::errors::DiagnosticResult;
use crate::diagnostics::registry::{RuleRegistry, ValidatorRegistry};
use crate::diagnostics::validator::{CoreSyntaxValidator, ValidationContext};
use crate::parser::tree::SyntaxTree;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use std::time::Instant;

/// Performance and validation execution metrics.
#[derive(Debug, Clone, Default)]
pub struct ValidationMetrics {
    /// Total validation runs performed.
    pub total_validations: u64,
    /// Count of cache hits.
    pub cache_hits: u64,
    /// Total diagnostics produced.
    pub diagnostics_generated: u64,
    /// Total duration spent validating (in milliseconds).
    pub total_duration_ms: u64,
}

/// The main Diagnostics & Validation Engine for GIC.
pub struct ValidationEngine {
    rule_registry: Arc<Mutex<RuleRegistry>>,
    validator_registry: Arc<Mutex<ValidatorRegistry>>,
    cache: Arc<Mutex<DiagnosticCache>>,
    metrics: Arc<Mutex<ValidationMetrics>>,
}

impl Default for ValidationEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl ValidationEngine {
    /// Creates a new `ValidationEngine` pre-configured with default syntax validators.
    pub fn new() -> Self {
        let mut validator_reg = ValidatorRegistry::new();
        validator_reg.register_global(CoreSyntaxValidator);

        Self {
            rule_registry: Arc::new(Mutex::new(RuleRegistry::new())),
            validator_registry: Arc::new(Mutex::new(validator_reg)),
            cache: Arc::new(Mutex::new(DiagnosticCache::default())),
            metrics: Arc::new(Mutex::new(ValidationMetrics::default())),
        }
    }

    /// Accesses the rule registry for rule configuration and registration.
    pub fn rule_registry(&self) -> std::sync::MutexGuard<'_, RuleRegistry> {
        self.rule_registry.lock().unwrap()
    }

    /// Accesses the validator registry.
    pub fn validator_registry(&self) -> std::sync::MutexGuard<'_, ValidatorRegistry> {
        self.validator_registry.lock().unwrap()
    }

    /// Accesses the diagnostic cache.
    pub fn cache(&self) -> std::sync::MutexGuard<'_, DiagnosticCache> {
        self.cache.lock().unwrap()
    }

    /// Returns a snapshot of validation engine metrics.
    pub fn metrics(&self) -> ValidationMetrics {
        self.metrics.lock().unwrap().clone()
    }

    /// Validates a parsed syntax tree and source document.
    ///
    /// Accepts:
    /// - `document_id`: Unique identifier for the document buffer.
    /// - `tree`: Parsed `SyntaxTree` output from Milestone 7 Parser.
    /// - `source_text`: Raw document source string.
    /// - `cancel_flag`: Optional cancellation signal.
    pub fn validate(
        &self,
        document_id: &str,
        tree: &SyntaxTree,
        source_text: &str,
        cancel_flag: Option<Arc<AtomicBool>>,
    ) -> DiagnosticResult<Vec<Diagnostic>> {
        let start_time = Instant::now();

        // 1. Check Cache
        {
            let cache = self.cache.lock().unwrap();
            if let Some(cached_diags) = cache.get(document_id, tree.source_hash) {
                let mut metrics = self.metrics.lock().unwrap();
                metrics.total_validations += 1;
                metrics.cache_hits += 1;
                return Ok(cached_diags.to_vec());
            }
        }

        // 2. Prepare Context
        let mut ctx = ValidationContext::new(source_text, tree.language.clone());
        if let Some(flag) = cancel_flag {
            ctx = ctx.with_cancel_flag(flag);
        }

        ctx.check_cancelled()?;

        let mut diagnostics = Vec::new();

        // 3. Execute Registered Validators
        let validators = {
            let val_reg = self.validator_registry.lock().unwrap();
            val_reg.get_validators(tree.language.clone())
        };

        for validator in validators {
            ctx.check_cancelled()?;
            let val_diags = validator.validate(tree, &ctx)?;
            diagnostics.extend(val_diags);
        }

        // 4. Execute Rule-based validation from RuleRegistry
        let active_rules = {
            let rule_reg = self.rule_registry.lock().unwrap();
            rule_reg.get_rules_for_language(tree.language.clone())
        };

        for rule in active_rules {
            ctx.check_cancelled()?;
            let rule_diags = rule.evaluate(tree, &ctx)?;
            diagnostics.extend(rule_diags);
        }

        // Deduplicate diagnostics by ID
        diagnostics.dedup_by(|a, b| a.id == b.id);

        // 5. Update Cache & Metrics
        {
            let mut cache = self.cache.lock().unwrap();
            cache.put(document_id, tree.source_hash, diagnostics.clone());
        }

        let elapsed = start_time.elapsed().as_millis() as u64;
        {
            let mut metrics = self.metrics.lock().unwrap();
            metrics.total_validations += 1;
            metrics.diagnostics_generated += diagnostics.len() as u64;
            metrics.total_duration_ms += elapsed;
        }

        Ok(diagnostics)
    }

    /// Fast refresh / clear cache for a specific document.
    pub fn refresh_document(&self, document_id: &str) {
        let mut cache = self.cache.lock().unwrap();
        cache.invalidate(document_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::language::LanguageId;
    use crate::parser::node::{NodeKind, SyntaxNode};
    use crate::parser::position::TextRange;

    #[test]
    fn test_engine_validation_flow() {
        let engine = ValidationEngine::new();

        let root = SyntaxNode::new(
            NodeKind::Document,
            "document",
            TextRange::empty(crate::parser::position::Position::zero()),
            Vec::new(),
        );
        let tree = SyntaxTree::new(LanguageId::Yaml, root, Vec::new(), Vec::new(), 987654, 100);

        let diags = engine
            .validate("test_doc.yaml", &tree, "key: value\n", None)
            .unwrap();
        assert!(diags.is_empty());

        let metrics = engine.metrics();
        assert_eq!(metrics.total_validations, 1);
        assert_eq!(metrics.cache_hits, 0);

        // Second validation should hit cache
        let cached_diags = engine
            .validate("test_doc.yaml", &tree, "key: value\n", None)
            .unwrap();
        assert!(cached_diags.is_empty());
        assert_eq!(engine.metrics().cache_hits, 1);
    }
}
