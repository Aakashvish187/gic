use std::sync::Arc;
use std::time::Instant;

use super::{
    action::DiagnosticAction, cache::RuleCache, context::EvaluationContext, errors::Result,
    metrics::RuleEngineMetrics, rule::Rule,
};

/// Evaluates a batch of rules against a given context.
pub struct RuleEvaluator {
    metrics: Arc<RuleEngineMetrics>,
    cache: Arc<RuleCache>,
}

impl RuleEvaluator {
    /// Creates a new RuleEvaluator.
    pub fn new(metrics: Arc<RuleEngineMetrics>, cache: Arc<RuleCache>) -> Self {
        Self { metrics, cache }
    }

    /// Evaluates the active rules against the context, utilizing cache and updating metrics.
    pub fn evaluate_batch(
        &self,
        ctx: &EvaluationContext,
        rules: Vec<Arc<dyn Rule>>,
    ) -> Result<Vec<DiagnosticAction>> {
        let mut all_actions = Vec::new();

        for rule in rules {
            let metadata = rule.metadata();

            // 1. Check Cache
            if let Some(cached) = self.cache.get(ctx.file_path, &metadata.id) {
                all_actions.extend(cached);
                self.metrics
                    .cache_hits
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                continue;
            }
            self.metrics
                .cache_misses
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

            // 2. Evaluate Rule
            let start = Instant::now();
            let result = rule.evaluate(ctx);
            let elapsed = start.elapsed().as_nanos() as u64;

            // 3. Process Result
            match result {
                Ok(actions) => {
                    let mut diags = Vec::new();
                    for action in actions {
                        if let Some(diag) = action.as_diagnostic() {
                            diags.push(diag.clone());
                        }
                    }

                    self.metrics
                        .record_execution(&metadata.id, elapsed, diags.len() as u64);

                    // Update cache for next time
                    self.cache
                        .insert(ctx.file_path, &metadata.id, diags.clone());

                    all_actions.extend(diags);
                }
                Err(e) => {
                    self.metrics.record_error(&metadata.id);
                    super::logger::RuleLogger::error(&format!(
                        "Rule {} failed: {}",
                        metadata.id, e
                    ));
                }
            }
        }

        Ok(all_actions)
    }
}
