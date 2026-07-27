use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use super::{
    errors::{Result, RuleEngineError},
    rule::Rule,
};

/// The centralized registry for managing rules across all languages and categories.
pub struct RuleRegistry {
    /// Thread-safe storage for registered rules, keyed by rule ID.
    rules: RwLock<HashMap<String, Arc<dyn Rule>>>,
}

impl Default for RuleRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl RuleRegistry {
    /// Creates a new, empty RuleRegistry.
    pub fn new() -> Self {
        Self {
            rules: RwLock::new(HashMap::new()),
        }
    }

    /// Registers a rule into the registry.
    ///
    /// # Errors
    /// Returns `RuleEngineError::DuplicateRule` if a rule with the same ID already exists.
    pub fn register(&self, rule: Arc<dyn Rule>) -> Result<()> {
        let mut lock = self.rules.write().unwrap();
        let id = rule.metadata().id.clone();

        if lock.contains_key(&id) {
            return Err(RuleEngineError::DuplicateRule(id));
        }

        lock.insert(id, rule);
        Ok(())
    }

    /// Loads multiple rules into the registry.
    pub fn load_batch(&self, rules: impl IntoIterator<Item = Arc<dyn Rule>>) -> Result<()> {
        let mut lock = self.rules.write().unwrap();
        for rule in rules {
            let id = rule.metadata().id.clone();
            if lock.contains_key(&id) {
                return Err(RuleEngineError::DuplicateRule(id));
            }
            lock.insert(id, rule);
        }
        Ok(())
    }

    /// Unregisters a rule by its ID.
    pub fn unregister(&self, id: &str) -> Result<()> {
        let mut lock = self.rules.write().unwrap();
        if lock.remove(id).is_some() {
            Ok(())
        } else {
            Err(RuleEngineError::RuleNotFound(id.to_string()))
        }
    }

    /// Retrieves a rule by its ID.
    pub fn get(&self, id: &str) -> Result<Arc<dyn Rule>> {
        let lock = self.rules.read().unwrap();
        lock.get(id)
            .cloned()
            .ok_or_else(|| RuleEngineError::RuleNotFound(id.to_string()))
    }

    /// Retrieves all registered rules.
    pub fn get_all(&self) -> Vec<Arc<dyn Rule>> {
        let lock = self.rules.read().unwrap();
        lock.values().cloned().collect()
    }

    /// Returns the number of rules in the registry.
    pub fn count(&self) -> usize {
        let lock = self.rules.read().unwrap();
        lock.len()
    }
}
