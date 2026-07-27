//! Incremental Validation and Formatting Cache for YAML.
//!
//! Provides a thread-safe, high-performance cache for storing parsed ASTs,
//! diagnostics, formatted strings, and folding ranges keyed by document content hash.

use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::RwLock;

use crate::yaml::folding::FoldingRange;
use crate::yaml::parser::YamlAST;
use crate::yaml::validator::YamlValidationDiagnostic;

/// Cached entry holding validation and formatting artifacts.
#[derive(Debug, Clone)]
pub struct YamlCacheEntry {
    /// Content hash of original source text.
    pub hash: u64,
    /// Parsed YAML AST.
    pub ast: YamlAST,
    /// Computed validation diagnostics.
    pub diagnostics: Vec<YamlValidationDiagnostic>,
    /// Formatted YAML output string.
    pub formatted_output: Option<String>,
    /// Computed code folding ranges.
    pub folding_ranges: Vec<FoldingRange>,
    /// Timestamp when cached.
    pub created_at_ms: u64,
}

/// Cache statistics metrics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct YamlCacheMetrics {
    pub hits: usize,
    pub misses: usize,
    pub entries_count: usize,
}

/// Thread-safe incremental YAML cache.
#[derive(Debug)]
pub struct YamlCache {
    entries: RwLock<HashMap<u64, YamlCacheEntry>>,
    capacity: usize,
    metrics: RwLock<YamlCacheMetrics>,
}

impl Default for YamlCache {
    fn default() -> Self {
        Self::new(100)
    }
}

impl YamlCache {
    /// Creates a new YamlCache with a specified capacity limit.
    pub fn new(capacity: usize) -> Self {
        Self {
            entries: RwLock::new(HashMap::new()),
            capacity,
            metrics: RwLock::new(YamlCacheMetrics::default()),
        }
    }

    /// Computes hash for raw YAML source text.
    pub fn compute_hash(source: &str) -> u64 {
        let mut hasher = DefaultHasher::new();
        source.hash(&mut hasher);
        hasher.finish()
    }

    /// Retrieves cached entry if present.
    pub fn get(&self, source: &str) -> Option<YamlCacheEntry> {
        let hash = Self::compute_hash(source);
        let guard = self.entries.read().ok()?;

        if let Some(entry) = guard.get(&hash) {
            if let Ok(mut m) = self.metrics.write() {
                m.hits += 1;
            }
            Some(entry.clone())
        } else {
            if let Ok(mut m) = self.metrics.write() {
                m.misses += 1;
            }
            None
        }
    }

    /// Inserts a new entry into the cache.
    pub fn put(
        &self,
        source: &str,
        ast: YamlAST,
        diagnostics: Vec<YamlValidationDiagnostic>,
        formatted_output: Option<String>,
        folding_ranges: Vec<FoldingRange>,
    ) {
        let hash = Self::compute_hash(source);
        let entry = YamlCacheEntry {
            hash,
            ast,
            diagnostics,
            formatted_output,
            folding_ranges,
            created_at_ms: 0,
        };

        if let Ok(mut guard) = self.entries.write() {
            if guard.len() >= self.capacity && !guard.contains_key(&hash) {
                // Simple eviction: remove one arbitrary entry when full
                if let Some(&first_key) = guard.keys().next() {
                    guard.remove(&first_key);
                }
            }
            guard.insert(hash, entry);

            if let Ok(mut m) = self.metrics.write() {
                m.entries_count = guard.len();
            }
        }
    }

    /// Clears all entries from the cache.
    pub fn clear(&self) {
        if let Ok(mut guard) = self.entries.write() {
            guard.clear();
        }
        if let Ok(mut m) = self.metrics.write() {
            m.entries_count = 0;
            m.hits = 0;
            m.misses = 0;
        }
    }

    /// Returns current cache metrics snapshot.
    pub fn metrics(&self) -> YamlCacheMetrics {
        self.metrics.read().map(|m| *m).unwrap_or_default()
    }
}
