use std::collections::HashMap;
use std::sync::RwLock;

use super::action::DiagnosticAction;

/// Manages caching of rule evaluation results to optimize performance for unchanged code regions.
#[derive(Default)]
pub struct RuleCache {
    /// Mapping of File Path -> (Rule ID -> Cached Diagnostics)
    cached_results: RwLock<HashMap<String, HashMap<String, Vec<DiagnosticAction>>>>,
}

impl RuleCache {
    /// Creates a new, empty cache.
    pub fn new() -> Self {
        Self::default()
    }

    /// Retrieves cached actions for a specific file and rule, if they exist.
    pub fn get(&self, file_path: &str, rule_id: &str) -> Option<Vec<DiagnosticAction>> {
        let lock = self.cached_results.read().unwrap();
        if let Some(file_cache) = lock.get(file_path) {
            return file_cache.get(rule_id).cloned();
        }
        None
    }

    /// Stores the evaluation actions for a rule against a specific file.
    pub fn insert(&self, file_path: &str, rule_id: &str, actions: Vec<DiagnosticAction>) {
        let mut lock = self.cached_results.write().unwrap();
        let file_cache = lock.entry(file_path.to_string()).or_default();
        file_cache.insert(rule_id.to_string(), actions);
    }

    /// Invalidates all cached entries for a specific file (e.g., when the file is modified).
    pub fn invalidate_file(&self, file_path: &str) {
        let mut lock = self.cached_results.write().unwrap();
        lock.remove(file_path);
    }

    /// Clears the entire cache.
    pub fn clear(&self) {
        let mut lock = self.cached_results.write().unwrap();
        lock.clear();
    }
}
