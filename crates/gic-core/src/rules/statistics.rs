/// Tracks statistics for an individual rule.
#[derive(Debug, Clone, Default)]
pub struct RuleStatistics {
    /// Number of times the rule has been executed.
    pub execution_count: u64,
    /// Number of times the rule was skipped (e.g., due to configuration or caching).
    pub skipped_count: u64,
    /// Number of violations found by this rule.
    pub violations_found: u64,
    /// Total time spent evaluating this rule (in nanoseconds).
    pub total_time_ns: u64,
    /// Number of times the rule encountered an internal error.
    pub error_count: u64,
}

impl RuleStatistics {
    /// Calculates the average execution time in nanoseconds.
    pub fn average_time_ns(&self) -> u64 {
        if self.execution_count == 0 {
            0
        } else {
            self.total_time_ns / self.execution_count
        }
    }
}
