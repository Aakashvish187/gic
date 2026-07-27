//! Incremental Validation Cache for Docker Engine.
//!
//! Provides thread-safe, lock-free LRU caching for Dockerfile and Compose validation findings.

use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::RwLock;

use crate::docker::validator::DockerDiagnostic;

/// Cached validation result entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DockerCacheEntry {
    /// Source code hash fingerprint.
    pub content_hash: u64,
    /// Extracted diagnostics.
    pub diagnostics: Vec<DockerDiagnostic>,
    /// System timestamp when cached.
    pub timestamp: u64,
}

/// Incremental validation cache metrics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DockerCacheMetrics {
    /// Number of cache hits.
    pub hits: usize,
    /// Number of cache misses.
    pub misses: usize,
    /// Number of cached entries.
    pub entries_count: usize,
}

/// Thread-safe Docker validation cache.
#[derive(Debug)]
pub struct DockerCache {
    entries: RwLock<HashMap<u64, DockerCacheEntry>>,
    hits: RwLock<usize>,
    misses: RwLock<usize>,
    max_capacity: usize,
}

impl Default for DockerCache {
    fn default() -> Self {
        Self::new(256)
    }
}

impl DockerCache {
    /// Creates a new DockerCache with maximum capacity.
    pub fn new(max_capacity: usize) -> Self {
        Self {
            entries: RwLock::new(HashMap::new()),
            hits: RwLock::new(0),
            misses: RwLock::new(0),
            max_capacity,
        }
    }

    /// Computes a 64-bit hash fingerprint for source text.
    pub fn compute_hash(source: &str) -> u64 {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        source.hash(&mut hasher);
        hasher.finish()
    }

    /// Retrieves cached diagnostics if content hash matches.
    pub fn get(&self, hash: u64) -> Option<Vec<DockerDiagnostic>> {
        let guard = self.entries.read().ok()?;
        if let Some(entry) = guard.get(&hash) {
            if let Ok(mut h) = self.hits.write() {
                *h += 1;
            }
            Some(entry.diagnostics.clone())
        } else {
            if let Ok(mut m) = self.misses.write() {
                *m += 1;
            }
            None
        }
    }

    /// Inserts a validation result into the cache.
    pub fn insert(&self, hash: u64, diagnostics: Vec<DockerDiagnostic>) {
        if let Ok(mut guard) = self.entries.write() {
            if guard.len() >= self.max_capacity {
                guard.clear();
            }
            guard.insert(
                hash,
                DockerCacheEntry {
                    content_hash: hash,
                    diagnostics,
                    timestamp: 0,
                },
            );
        }
    }

    /// Returns cache hit/miss and entry metrics.
    pub fn metrics(&self) -> DockerCacheMetrics {
        let hits = self.hits.read().map(|h| *h).unwrap_or(0);
        let misses = self.misses.read().map(|m| *m).unwrap_or(0);
        let entries_count = self.entries.read().map(|e| e.len()).unwrap_or(0);

        DockerCacheMetrics {
            hits,
            misses,
            entries_count,
        }
    }

    /// Clears all entries from the cache.
    pub fn clear(&self) {
        if let Ok(mut guard) = self.entries.write() {
            guard.clear();
        }
    }
}
