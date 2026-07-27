//! DiagnosticCache unit and capacity eviction tests.

use crate::diagnostics::cache::DiagnosticCache;
use crate::diagnostics::diagnostic::Diagnostic;
use crate::diagnostics::position::DiagnosticPosition;
use crate::diagnostics::range::DiagnosticRange;
use crate::diagnostics::severity::DiagnosticLevel;
use crate::parser::language::LanguageId;

#[test]
fn test_cache_hit_and_eviction() {
    let mut cache = DiagnosticCache::new(2);

    let p1 = DiagnosticPosition::new(1, 1, 0);
    let range = DiagnosticRange::new(p1, p1);
    let diag = Diagnostic::new(
        DiagnosticLevel::Error,
        "Error",
        range,
        "Rule",
        LanguageId::Yaml,
    );

    cache.put("doc1", 100, vec![diag.clone()]);
    cache.put("doc2", 200, vec![diag.clone()]);
    assert_eq!(cache.len(), 2);

    // Evict oldest (doc1) by putting doc3
    cache.put("doc3", 300, vec![diag]);
    assert_eq!(cache.len(), 2);

    assert!(cache.get("doc2", 200).is_some());
    assert!(cache.get("doc3", 300).is_some());
    assert!(cache.get("doc1", 100).is_none());
}

#[test]
fn test_dirty_region_filtering() {
    let cache = DiagnosticCache::new(10);

    let p1 = DiagnosticPosition::new(1, 1, 0);
    let p2 = DiagnosticPosition::new(1, 5, 4);
    let r1 = DiagnosticRange::new(p1, p2); // Line 1

    let p3 = DiagnosticPosition::new(10, 1, 100);
    let p4 = DiagnosticPosition::new(10, 5, 104);
    let r2 = DiagnosticRange::new(p3, p4); // Line 10

    let diag1 = Diagnostic::new(
        DiagnosticLevel::Warning,
        "Warn 1",
        r1,
        "Rule1",
        LanguageId::Yaml,
    );
    let diag2 = Diagnostic::new(
        DiagnosticLevel::Warning,
        "Warn 2",
        r2,
        "Rule2",
        LanguageId::Yaml,
    );

    let mut mutable_cache = cache;
    mutable_cache.put("doc_dirty", 555, vec![diag1, diag2]);

    // Dirty region on Line 1 only
    let dirty = vec![r1];
    let retained = mutable_cache
        .filter_dirty_diagnostics("doc_dirty", &dirty)
        .unwrap();

    assert_eq!(retained.len(), 1);
    assert_eq!(retained[0].rule_name, "Rule2");
}
