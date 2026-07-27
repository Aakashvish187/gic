use super::statistics::RuleStatistics;
use std::collections::HashMap;
use std::sync::{
    atomic::{AtomicU64, Ordering},
    RwLock,
};

/// Global metrics for the Universal Rule Engine.
#[derive(Default)]
pub struct RuleEngineMetrics {
    /// Per-rule statistics, keyed by Rule ID.
    rule_stats: RwLock<HashMap<String, RuleStatistics>>,
    /// Total number of rule evaluations across all rules.
    pub total_evaluations: AtomicU64,
    /// Cache hits.
    pub cache_hits: AtomicU64,
    /// Cache misses.
    pub cache_misses: AtomicU64,
}

impl RuleEngineMetrics {
    pub fn new() -> Self {
        Self::default()
    }

    /// Records an execution for a specific rule.
    pub fn record_execution(&self, rule_id: &str, time_ns: u64, violations: u64) {
        self.total_evaluations.fetch_add(1, Ordering::Relaxed);
        let mut stats = self.rule_stats.write().unwrap();
        let rule_stat = stats
            .entry(rule_id.to_string())
            .or_default();
        rule_stat.execution_count += 1;
        rule_stat.total_time_ns += time_ns;
        rule_stat.violations_found += violations;
    }

    /// Records a skipped execution for a rule.
    pub fn record_skip(&self, rule_id: &str) {
        let mut stats = self.rule_stats.write().unwrap();
        let rule_stat = stats
            .entry(rule_id.to_string())
            .or_default();
        rule_stat.skipped_count += 1;
    }

    /// Records an error encountered by a rule.
    pub fn record_error(&self, rule_id: &str) {
        let mut stats = self.rule_stats.write().unwrap();
        let rule_stat = stats
            .entry(rule_id.to_string())
            .or_default();
        rule_stat.error_count += 1;
    }

    /// Retrieves a snapshot of the current metrics.
    pub fn snapshot_stats(&self) -> HashMap<String, RuleStatistics> {
        self.rule_stats.read().unwrap().clone()
    }
}
