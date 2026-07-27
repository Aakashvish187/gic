use serde::{Deserialize, Serialize};

/// Remembers recent search queries and replacement strings with MRU ordering and maximum capacity limits.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SearchHistory {
    recent_queries: Vec<String>,
    recent_replaces: Vec<String>,
    max_size: usize,
}

impl Default for SearchHistory {
    fn default() -> Self {
        Self::new(50)
    }
}

impl SearchHistory {
    /// Creates a new `SearchHistory` with specified maximum capacity.
    pub fn new(max_size: usize) -> Self {
        Self {
            recent_queries: Vec::new(),
            recent_replaces: Vec::new(),
            max_size: max_size.max(1),
        }
    }

    /// Adds a query string to history. Moves existing query to top if present.
    pub fn add_query(&mut self, query: &str) {
        if query.trim().is_empty() {
            return;
        }

        self.recent_queries.retain(|q| q != query);
        self.recent_queries.insert(0, query.to_string());

        if self.recent_queries.len() > self.max_size {
            self.recent_queries.truncate(self.max_size);
        }
    }

    /// Adds a replacement string to history. Moves existing replace string to top if present.
    pub fn add_replace(&mut self, replace: &str) {
        self.recent_replaces.retain(|r| r != replace);
        self.recent_replaces.insert(0, replace.to_string());

        if self.recent_replaces.len() > self.max_size {
            self.recent_replaces.truncate(self.max_size);
        }
    }

    /// Returns slice of recent search queries in MRU order.
    pub fn queries(&self) -> &[String] {
        &self.recent_queries
    }

    /// Returns slice of recent replacement strings in MRU order.
    pub fn replaces(&self) -> &[String] {
        &self.recent_replaces
    }

    /// Clears search queries history.
    pub fn clear_queries(&mut self) {
        self.recent_queries.clear();
    }

    /// Clears replacement history.
    pub fn clear_replaces(&mut self) {
        self.recent_replaces.clear();
    }

    /// Clears both query and replacement histories.
    pub fn clear_all(&mut self) {
        self.clear_queries();
        self.clear_replaces();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_history() {
        let mut history = SearchHistory::new(3);

        history.add_query("cat");
        history.add_query("dog");
        history.add_query("bird");
        assert_eq!(history.queries(), &["bird", "dog", "cat"]);

        // Push 4th query -> truncates oldest ("cat")
        history.add_query("fish");
        assert_eq!(history.queries(), &["fish", "bird", "dog"]);

        // Re-adding existing query brings it to top
        history.add_query("dog");
        assert_eq!(history.queries(), &["dog", "fish", "bird"]);
    }
}
