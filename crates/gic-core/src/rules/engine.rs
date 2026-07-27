use std::sync::Arc;

use super::{
    action::DiagnosticAction, cache::RuleCache, context::EvaluationContext, errors::Result,
    evaluator::RuleEvaluator, metrics::RuleEngineMetrics, registry::RuleRegistry,
    scheduler::RuleScheduler,
};

/// The Universal Rule Engine.
/// This acts as the facade for language engines to register and evaluate rules.
pub struct UniversalRuleEngine {
    pub registry: Arc<RuleRegistry>,
    pub metrics: Arc<RuleEngineMetrics>,
    pub cache: Arc<RuleCache>,
}

impl Default for UniversalRuleEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl UniversalRuleEngine {
    /// Constructs a new instance of the Universal Rule Engine.
    pub fn new() -> Self {
        Self {
            registry: Arc::new(RuleRegistry::new()),
            metrics: Arc::new(RuleEngineMetrics::new()),
            cache: Arc::new(RuleCache::new()),
        }
    }

    /// Evaluates a single file using the provided context.
    /// Returns a list of generated diagnostic actions (which may include quick fixes).
    pub fn evaluate_file(&self, ctx: &EvaluationContext) -> Result<Vec<DiagnosticAction>> {
        // 1. Scheduler filters and prioritizes rules based on settings.
        let scheduler = RuleScheduler::new(self.registry.clone());
        let active_rules = scheduler.get_active_rules(ctx.settings);

        // 2. Evaluator runs the active rules, utilizing the cache.
        let evaluator = RuleEvaluator::new(self.metrics.clone(), self.cache.clone());
        evaluator.evaluate_batch(ctx, active_rules)
    }

    /// Invalidates the cache for a specific file when it is edited or modified.
    pub fn invalidate_file_cache(&self, file_path: &str) {
        self.cache.invalidate_file(file_path);
    }
}
