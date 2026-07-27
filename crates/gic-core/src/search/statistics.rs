use serde::{Deserialize, Serialize};

/// Performance and diagnostic telemetry metrics for search operations.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct SearchStatistics {
    /// Total matches discovered in last search execution.
    pub total_matches: usize,
    /// 0-indexed position of active match selection.
    pub current_match_index: Option<usize>,
    /// Search execution duration in microseconds ($\mu s$).
    pub search_duration_us: u128,
    /// Cumulative count of replacements executed.
    pub replace_count: usize,
    /// Active query text if search is active.
    pub current_query: Option<String>,
    /// True if search panel/query is actively executing.
    pub is_active: bool,
}

impl SearchStatistics {
    /// Creates a new empty `SearchStatistics`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Resets search statistics counters.
    pub fn reset(&mut self) {
        self.total_matches = 0;
        self.current_match_index = None;
        self.search_duration_us = 0;
        self.replace_count = 0;
        self.current_query = None;
        self.is_active = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_statistics() {
        let mut stats = SearchStatistics::new();
        assert_eq!(stats.total_matches, 0);
        assert!(!stats.is_active);

        stats.total_matches = 42;
        stats.search_duration_us = 150;
        stats.is_active = true;

        assert_eq!(stats.total_matches, 42);
        assert_eq!(stats.search_duration_us, 150);

        stats.reset();
        assert_eq!(stats.total_matches, 0);
    }
}
