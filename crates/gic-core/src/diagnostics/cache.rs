//! Diagnostic result caching, dirty region invalidation, and incremental validation storage.

use crate::diagnostics::diagnostic::Diagnostic;
use crate::diagnostics::range::DiagnosticRange;
use std::collections::HashMap;
use std::time::Instant;

/// Cached entry containing diagnostics generated for a specific document snapshot hash.
#[derive(Debug, Clone)]
pub struct DiagnosticCacheEntry {
    /// Hash of the source document content.
    pub source_hash: u64,
    /// Cached diagnostic items.
    pub diagnostics: Vec<Diagnostic>,
    /// Cache creation timestamp.
    pub created_at: Instant,
}

/// In-memory cache for document validation results enabling fast re-validation.
pub struct DiagnosticCache {
    entries: HashMap<String, DiagnosticCacheEntry>,
    max_entries: usize,
}

impl Default for DiagnosticCache {
    fn default() -> Self {
        Self::new(100)
    }
}

impl DiagnosticCache {
    /// Creates a new `DiagnosticCache` with maximum capacity.
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: HashMap::new(),
            max_entries,
        }
    }

    /// Retrieves cached diagnostics if the `source_hash` matches.
    pub fn get(&self, document_id: &str, source_hash: u64) -> Option<&[Diagnostic]> {
        if let Some(entry) = self.entries.get(document_id) {
            if entry.source_hash == source_hash {
                return Some(&entry.diagnostics);
            }
        }
        None
    }

    /// Stores diagnostic results in the cache.
    pub fn put(
        &mut self,
        document_id: impl Into<String>,
        source_hash: u64,
        diagnostics: Vec<Diagnostic>,
    ) {
        let doc_id = document_id.into();
        if self.entries.len() >= self.max_entries && !self.entries.contains_key(&doc_id) {
            // Evict oldest entry if capacity exceeded
            if let Some(oldest_key) = self
                .entries
                .iter()
                .min_by_key(|(_, v)| v.created_at)
                .map(|(k, _)| k.clone())
            {
                self.entries.remove(&oldest_key);
            }
        }

        self.entries.insert(
            doc_id,
            DiagnosticCacheEntry {
                source_hash,
                diagnostics,
                created_at: Instant::now(),
            },
        );
    }

    /// Invalidates cache entry for a document.
    pub fn invalidate(&mut self, document_id: &str) {
        self.entries.remove(document_id);
    }

    /// Filter cached diagnostics by removing items that fall inside specified dirty regions.
    pub fn filter_dirty_diagnostics(
        &self,
        document_id: &str,
        dirty_regions: &[DiagnosticRange],
    ) -> Option<Vec<Diagnostic>> {
        let entry = self.entries.get(document_id)?;

        let retained: Vec<Diagnostic> = entry
            .diagnostics
            .iter()
            .filter(|diag| {
                !dirty_regions.iter().any(|dirty| {
                    dirty.intersects(&diag.range) || dirty.contains_position(diag.range.start)
                })
            })
            .cloned()
            .collect();

        Some(retained)
    }

    /// Clears all entries from the cache.
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Returns current number of cached entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns `true` if cache is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostics::position::DiagnosticPosition;
    use crate::diagnostics::severity::DiagnosticLevel;
    use crate::parser::language::LanguageId;

    #[test]
    fn test_diagnostic_cache_operations() {
        let mut cache = DiagnosticCache::new(5);
        let p1 = DiagnosticPosition::new(1, 1, 0);
        let range = DiagnosticRange::new(p1, p1);
        let diag = Diagnostic::new(
            DiagnosticLevel::Error,
            "Error",
            range,
            "Rule1",
            LanguageId::Yaml,
        );

        cache.put("doc1", 12345, vec![diag]);
        assert_eq!(cache.len(), 1);
        assert!(cache.get("doc1", 12345).is_some());
        assert!(cache.get("doc1", 99999).is_none());

        cache.invalidate("doc1");
        assert!(cache.get("doc1", 12345).is_none());
    }
}
