//! Incremental Validation Cache for Kubernetes Manifests.

use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::RwLock;

use crate::kubernetes::resource_detector::K8sResource;
use crate::kubernetes::validator::K8sDiagnostic;

/// Cache entry storing parsed resources and computed diagnostics.
#[derive(Debug, Clone)]
pub struct K8sCacheEntry {
    pub hash: u64,
    pub resources: Vec<K8sResource>,
    pub diagnostics: Vec<K8sDiagnostic>,
}

/// Cache statistics metrics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct K8sCacheMetrics {
    pub hits: usize,
    pub misses: usize,
    pub entries_count: usize,
}

/// Thread-safe incremental Kubernetes validation cache.
#[derive(Debug)]
pub struct K8sCache {
    entries: RwLock<HashMap<u64, K8sCacheEntry>>,
    capacity: usize,
    metrics: RwLock<K8sCacheMetrics>,
}

impl Default for K8sCache {
    fn default() -> Self {
        Self::new(100)
    }
}

impl K8sCache {
    pub fn new(capacity: usize) -> Self {
        Self {
            entries: RwLock::new(HashMap::new()),
            capacity,
            metrics: RwLock::new(K8sCacheMetrics::default()),
        }
    }

    pub fn compute_hash(source: &str) -> u64 {
        let mut hasher = DefaultHasher::new();
        source.hash(&mut hasher);
        hasher.finish()
    }

    pub fn get(&self, source: &str) -> Option<K8sCacheEntry> {
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

    pub fn put(&self, source: &str, resources: Vec<K8sResource>, diagnostics: Vec<K8sDiagnostic>) {
        let hash = Self::compute_hash(source);
        let entry = K8sCacheEntry {
            hash,
            resources,
            diagnostics,
        };

        if let Ok(mut guard) = self.entries.write() {
            if guard.len() >= self.capacity && !guard.contains_key(&hash) {
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

    pub fn metrics(&self) -> K8sCacheMetrics {
        self.metrics.read().map(|m| *m).unwrap_or_default()
    }
}
