//! Thread-safe syntax tree caching mechanism.

use crate::parser::tree::SyntaxTree;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Metrics recording cache performance statistics.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct CacheMetrics {
    pub hits: usize,
    pub misses: usize,
    pub evictions: usize,
    pub entries: usize,
}

#[derive(Debug, Clone)]
struct CacheEntry {
    source_hash: u64,
    tree: SyntaxTree,
}

/// Thread-safe LRU-capable parse tree cache.
#[derive(Debug, Clone)]
pub struct ParseCache {
    capacity: usize,
    entries: Arc<RwLock<HashMap<String, CacheEntry>>>,
    metrics: Arc<RwLock<CacheMetrics>>,
}

impl ParseCache {
    /// Creates a new `ParseCache` with the given capacity.
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            entries: Arc::new(RwLock::new(HashMap::new())),
            metrics: Arc::new(RwLock::new(CacheMetrics::default())),
        }
    }

    /// Looks up a cached `SyntaxTree` matching the document key and current source hash.
    pub fn get(&self, key: &str, source_hash: u64) -> Option<SyntaxTree> {
        let entries = self.entries.read().unwrap();
        let mut metrics = self.metrics.write().unwrap();

        if let Some(entry) = entries.get(key) {
            if entry.source_hash == source_hash {
                metrics.hits += 1;
                return Some(entry.tree.clone());
            }
        }

        metrics.misses += 1;
        None
    }

    /// Stores a parsed `SyntaxTree` in the cache.
    pub fn insert(&self, key: impl Into<String>, tree: SyntaxTree) {
        let key_str = key.into();
        let source_hash = tree.source_hash;
        let mut entries = self.entries.write().unwrap();
        let mut metrics = self.metrics.write().unwrap();

        if entries.len() >= self.capacity && !entries.contains_key(&key_str) {
            // Evict an arbitrary entry when capacity is reached
            if let Some(evict_key) = entries.keys().next().cloned() {
                entries.remove(&evict_key);
                metrics.evictions += 1;
            }
        }

        entries.insert(key_str, CacheEntry { source_hash, tree });
        metrics.entries = entries.len();
    }

    /// Clears all entries from the cache.
    pub fn clear(&self) {
        let mut entries = self.entries.write().unwrap();
        let mut metrics = self.metrics.write().unwrap();
        entries.clear();
        metrics.entries = 0;
    }

    /// Returns performance metrics.
    pub fn metrics(&self) -> CacheMetrics {
        *self.metrics.read().unwrap()
    }
}

impl Default for ParseCache {
    fn default() -> Self {
        Self::new(100)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::language::LanguageId;
    use crate::parser::node::{NodeKind, SyntaxNode};
    use crate::parser::position::{Position, TextRange};

    #[test]
    fn test_parse_cache_hit_and_miss() {
        let cache = ParseCache::new(5);
        let node = SyntaxNode::new(
            NodeKind::Document,
            "root",
            TextRange::new(Position::zero(), Position::zero()),
            vec![],
        );
        let tree = SyntaxTree::new(LanguageId::PlainText, node, vec![], vec![], 12345, 10);

        cache.insert("doc1", tree.clone());

        // Exact hash match -> Hit
        assert!(cache.get("doc1", 12345).is_some());
        // Mismatched hash -> Miss
        assert!(cache.get("doc1", 99999).is_none());

        let metrics = cache.metrics();
        assert_eq!(metrics.hits, 1);
        assert_eq!(metrics.misses, 1);
    }
}
