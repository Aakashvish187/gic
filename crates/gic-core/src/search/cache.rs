use crate::search::matcher::SearchMatch;
use crate::search::options::SearchOptions;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

/// Hash key identifying a search operation.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct CacheKey {
    query: String,
    options: SearchOptions,
    buffer_hash: u64,
}

/// Caches repeated search match results to eliminate redundant computation.
#[derive(Debug, Clone, Default)]
pub struct SearchCache {
    entries: HashMap<CacheKey, Vec<SearchMatch>>,
    capacity: usize,
}

impl SearchCache {
    /// Creates a new `SearchCache` with specified max entries.
    pub fn new(capacity: usize) -> Self {
        Self {
            entries: HashMap::with_capacity(capacity),
            capacity: capacity.max(1),
        }
    }

    /// Computes hash for buffer line contents.
    pub fn hash_buffer(lines: &[String]) -> u64 {
        let mut hasher = DefaultHasher::new();
        lines.len().hash(&mut hasher);
        for line in lines {
            line.hash(&mut hasher);
        }
        hasher.finish()
    }

    /// Retrieves cached match vector if key matches.
    pub fn get(
        &self,
        query: &str,
        options: &SearchOptions,
        buffer_hash: u64,
    ) -> Option<&Vec<SearchMatch>> {
        let key = CacheKey {
            query: query.to_string(),
            options: options.clone(),
            buffer_hash,
        };
        self.entries.get(&key)
    }

    /// Stores match result in cache. Truncates if capacity exceeded.
    pub fn put(
        &mut self,
        query: &str,
        options: SearchOptions,
        buffer_hash: u64,
        matches: Vec<SearchMatch>,
    ) {
        if self.entries.len() >= self.capacity {
            self.entries.clear(); // Clear cache when capacity limit reached
        }

        let key = CacheKey {
            query: query.to_string(),
            options,
            buffer_hash,
        };

        self.entries.insert(key, matches);
    }

    /// Invalidates all cached search results.
    pub fn invalidate(&mut self) {
        self.entries.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_cache() {
        let mut cache = SearchCache::new(5);
        let lines = vec!["line 1".to_string(), "line 2".to_string()];
        let hash = SearchCache::hash_buffer(&lines);
        let opts = SearchOptions::default();

        assert!(cache.get("test", &opts, hash).is_none());

        cache.put("test", opts.clone(), hash, vec![]);
        assert!(cache.get("test", &opts, hash).is_some());

        cache.invalidate();
        assert!(cache.get("test", &opts, hash).is_none());
    }
}
