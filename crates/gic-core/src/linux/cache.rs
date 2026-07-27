//! Incremental Validation Cache for Linux Intelligence Engine.
//!
//! Stores validation results indexed by 64-bit content hashes using LRU cache.

use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::RwLock;

use crate::linux::validator::LinuxDiagnostic;

#[derive(Debug)]
pub struct LinuxCache {
    capacity: usize,
    entries: RwLock<HashMap<u64, Vec<LinuxDiagnostic>>>,
}

impl Default for LinuxCache {
    fn default() -> Self {
        Self::new(256)
    }
}

impl LinuxCache {
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            entries: RwLock::new(HashMap::with_capacity(capacity)),
        }
    }

    pub fn compute_hash(source: &str) -> u64 {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        source.hash(&mut hasher);
        hasher.finish()
    }

    pub fn get(&self, hash: u64) -> Option<Vec<LinuxDiagnostic>> {
        let guard = self.entries.read().ok()?;
        guard.get(&hash).cloned()
    }

    pub fn insert(&self, hash: u64, diagnostics: Vec<LinuxDiagnostic>) {
        if let Ok(mut guard) = self.entries.write() {
            if guard.len() >= self.capacity {
                guard.clear();
            }
            guard.insert(hash, diagnostics);
        }
    }

    pub fn clear(&self) {
        if let Ok(mut guard) = self.entries.write() {
            guard.clear();
        }
    }

    pub fn len(&self) -> usize {
        self.entries.read().map(|g| g.len()).unwrap_or(0)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
