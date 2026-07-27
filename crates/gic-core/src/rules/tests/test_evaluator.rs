use crate::parser::TextRange;
use crate::rules::*;
use std::sync::Arc;

struct FailingRule {
    metadata: RuleMetadata,
}

impl FailingRule {
    fn new(id: &str) -> Self {
        Self {
            metadata: RuleMetadata {
                id: id.to_string(),
                name: "Failing".into(),
                short_description: "".into(),
                long_description: "".into(),
                language: "test".into(),
                category: RuleCategory::Support,
                severity: RuleSeverity::Error,
                priority: RulePriority::Medium,
                version: "1.0".into(),
                author: "Test".into(),
                documentation: RuleDocumentation::default(),
                tags: RuleTags::default(),
                enabled: true,
            },
        }
    }
}

impl Rule for FailingRule {
    fn metadata(&self) -> &RuleMetadata {
        &self.metadata
    }

    fn evaluate(&self, _ctx: &dyn RuleContext) -> Result<Vec<Box<dyn RuleAction>>> {
        let action = DiagnosticAction::new(
            RuleSeverity::Error,
            "Failed!",
            TextRange::empty(crate::parser::Position::new(0, 0, 0)),
        );
        Ok(vec![Box::new(action)])
    }
}

#[test]
fn test_evaluator_execution_and_cache() {
    let metrics = Arc::new(RuleEngineMetrics::new());
    let cache = Arc::new(RuleCache::new());
    let evaluator = RuleEvaluator::new(metrics.clone(), cache.clone());

    let settings = WorkspaceSettings::new();
    let ctx = EvaluationContext {
        file_path: "test.rs",
        root_node: None,
        settings: &settings,
        language: "rust",
    };

    let rule = Arc::new(FailingRule::new("TEST_ERR"));

    // First evaluation (Cache miss)
    let actions = evaluator.evaluate_batch(&ctx, vec![rule.clone()]).unwrap();
    assert_eq!(actions.len(), 1);
    assert_eq!(
        metrics
            .cache_misses
            .load(std::sync::atomic::Ordering::Relaxed),
        1
    );
    assert_eq!(
        metrics
            .cache_hits
            .load(std::sync::atomic::Ordering::Relaxed),
        0
    );

    // Second evaluation (Cache hit)
    let actions_cached = evaluator.evaluate_batch(&ctx, vec![rule.clone()]).unwrap();
    assert_eq!(actions_cached.len(), 1);
    assert_eq!(
        metrics
            .cache_misses
            .load(std::sync::atomic::Ordering::Relaxed),
        1
    );
    assert_eq!(
        metrics
            .cache_hits
            .load(std::sync::atomic::Ordering::Relaxed),
        1
    );
}
