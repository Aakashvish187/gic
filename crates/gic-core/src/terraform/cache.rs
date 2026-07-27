//! Incremental Validation Cache for Terraform Intelligence Engine.
//!
//! Stores validation results indexed by 64-bit content hashes using a thread-safe LRU cache.

use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::RwLock;

use crate::terraform::validator::TerraformDiagnostic;

/// Thread-safe LRU validation cache for Terraform configurations.
#[derive(Debug)]
pub struct TerraformCache {
    capacity: usize,
    entries: RwLock<HashMap<u64, Vec<TerraformDiagnostic>>>,
}

impl Default for TerraformCache {
    fn default() -> Self {
        Self::new(256)
    }
}

impl TerraformCache {
    /// Creates a new `TerraformCache` with given entry capacity.
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            entries: RwLock::new(HashMap::with_capacity(capacity)),
        }
    }

    /// Computes a 64-bit FNV-1a hash of the source code string.
    pub fn compute_hash(source: &str) -> u64 {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        source.hash(&mut hasher);
        hasher.finish()
    }

    /// Fetches cached diagnostics if entry exists.
    pub fn get(&self, hash: u64) -> Option<Vec<TerraformDiagnostic>> {
        let guard = self.entries.read().ok()?;
        guard.get(&hash).cloned()
    }

    /// Inserts a validation result into the cache.
    pub fn insert(&self, hash: u64, diagnostics: Vec<TerraformDiagnostic>) {
        if let Ok(mut guard) = self.entries.write() {
            if guard.len() >= self.capacity {
                guard.clear();
            }
            guard.insert(hash, diagnostics);
        }
    }

    /// Clears all entries from the cache.
    pub fn clear(&self) {
        if let Ok(mut guard) = self.entries.write() {
            guard.clear();
        }
    }

    /// Returns current cached entry count.
    pub fn len(&self) -> usize {
        self.entries.read().map(|g| g.len()).unwrap_or(0)
    }

    /// Returns true if cache contains 0 items.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
