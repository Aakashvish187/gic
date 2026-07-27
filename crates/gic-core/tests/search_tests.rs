use gic_core::buffer::{CursorPosition, TextBuffer};
use gic_core::search::{SearchEngine, SearchError, SearchMode, SearchOptions};

#[test]
fn test_simple_and_case_sensitive_search() {
    let mut engine = SearchEngine::default();
    let buffer = TextBuffer::from_str("Rust is fast.\nFAST code in rust.\nRustaceans love RUST.");

    // Default: Case-insensitive search for "rust"
    let matches = engine.search(&buffer, "rust").unwrap();
    assert_eq!(matches.len(), 4);

    // Case-sensitive search for "Rust"
    engine.set_options(SearchOptions::new().with_case_sensitive(true));
    let cs_matches = engine.search(&buffer, "Rust").unwrap();
    assert_eq!(cs_matches.len(), 2);
    assert_eq!(cs_matches[0].range.start, CursorPosition::new(0, 0));
    assert_eq!(cs_matches[1].range.start, CursorPosition::new(2, 0));
}

#[test]
fn test_whole_word_prefix_suffix_search() {
    let mut engine = SearchEngine::default();
    let buffer = TextBuffer::from_str("cat concatenate catalog copycat cat");

    // Whole word matching
    engine.set_options(SearchOptions::new().with_mode(SearchMode::WholeWord));
    let matches = engine.search(&buffer, "cat").unwrap();
    assert_eq!(matches.len(), 2);
    assert_eq!(matches[0].range.start, CursorPosition::new(0, 0));
    assert_eq!(matches[1].range.start, CursorPosition::new(0, 32));

    // Prefix matching
    engine.set_options(SearchOptions::new().with_mode(SearchMode::Prefix));
    let prefix_matches = engine.search(&buffer, "cat").unwrap();
    assert_eq!(prefix_matches.len(), 3); // "cat", "catalog", "cat"

    // Suffix matching
    engine.set_options(SearchOptions::new().with_mode(SearchMode::Suffix));
    let suffix_matches = engine.search(&buffer, "cat").unwrap();
    assert_eq!(suffix_matches.len(), 3); // "cat", "copycat", "cat"
}

#[test]
fn test_unicode_and_utf8_char_indexing() {
    let mut engine = SearchEngine::default();
    // Multi-byte UTF-8 characters and emojis
    let buffer = TextBuffer::from_str("αβγ 🚀 launch\n🚀 rocket 🚀");

    let matches = engine.search(&buffer, "🚀").unwrap();
    assert_eq!(matches.len(), 3);
    assert_eq!(matches[0].range.start, CursorPosition::new(0, 4));
    assert_eq!(matches[0].range.end, CursorPosition::new(0, 5));
    assert_eq!(matches[1].range.start, CursorPosition::new(1, 0));
    assert_eq!(matches[2].range.start, CursorPosition::new(1, 9));

    let greek_matches = engine.search(&buffer, "αβγ").unwrap();
    assert_eq!(greek_matches.len(), 1);
    assert_eq!(greek_matches[0].range.start, CursorPosition::new(0, 0));
    assert_eq!(greek_matches[0].range.end, CursorPosition::new(0, 3));
}

#[test]
fn test_empty_query_and_no_match_handling() {
    let mut engine = SearchEngine::default();
    let buffer = TextBuffer::from_str("Some sample text");

    assert_eq!(engine.search(&buffer, ""), Err(SearchError::EmptyQuery));

    let matches = engine.search(&buffer, "nonexistent").unwrap();
    assert!(matches.is_empty());
    assert_eq!(engine.total_matches(), 0);
    assert_eq!(engine.current_match(), None);
}

#[test]
fn test_navigation_and_wrap_around() {
    let mut engine = SearchEngine::default();
    let buffer = TextBuffer::from_str("match1\nmatch2\nmatch3");

    engine.search(&buffer, "match").unwrap();
    assert_eq!(engine.total_matches(), 3);
    assert_eq!(
        engine.current_match().unwrap().range.start,
        CursorPosition::new(0, 0)
    );

    // Next match
    assert_eq!(
        engine.next_match().unwrap().range.start,
        CursorPosition::new(1, 0)
    );
    assert_eq!(
        engine.next_match().unwrap().range.start,
        CursorPosition::new(2, 0)
    );

    // Wrap around to top
    assert_eq!(
        engine.next_match().unwrap().range.start,
        CursorPosition::new(0, 0)
    );

    // Previous match wrap around to bottom
    assert_eq!(
        engine.previous_match().unwrap().range.start,
        CursorPosition::new(2, 0)
    );

    // Jump to match index 1
    assert_eq!(
        engine.jump_to_match(1).unwrap().range.start,
        CursorPosition::new(1, 0)
    );
}

#[test]
fn test_cursor_synchronization() {
    let mut engine = SearchEngine::default();
    let buffer = TextBuffer::from_str("row 0\nrow 1 match\nrow 2\nrow 3 match");

    engine.search(&buffer, "match").unwrap();
    assert_eq!(engine.total_matches(), 2);

    // Sync cursor at row 2, col 0 -> next match is row 3
    engine.sync_cursor(CursorPosition::new(2, 0));
    assert_eq!(
        engine.current_match().unwrap().range.start,
        CursorPosition::new(3, 6)
    );
}

#[test]
fn test_viewport_highlights() {
    let mut engine = SearchEngine::default();
    let mut lines = Vec::new();
    for i in 0..50 {
        lines.push(format!("line {} target", i));
    }
    let buffer = TextBuffer::from_lines(lines);

    engine.search(&buffer, "target").unwrap();
    assert_eq!(engine.total_matches(), 50);

    // Compute highlights for viewport rows 10..=15
    let hl = engine.get_highlights(10, 15);
    assert_eq!(hl.len(), 6); // Rows 10, 11, 12, 13, 14, 15
}

#[test]
fn test_replace_current_and_replace_all() {
    let mut engine = SearchEngine::default();
    let mut buffer = TextBuffer::from_str("foo bar foo baz foo");

    engine.search(&buffer, "foo").unwrap();
    assert_eq!(engine.total_matches(), 3);

    // Replace current (first occurrence "foo" -> "qux")
    let res = engine.replace_current(&mut buffer, "qux").unwrap();
    assert_eq!(res.replacements_count, 1);
    assert_eq!(buffer.text(), "qux bar foo baz foo");

    // Replace all remaining "foo" -> "qux"
    let res_all = engine.replace_all(&mut buffer, "qux").unwrap();
    assert_eq!(res_all.replacements_count, 2);
    assert_eq!(buffer.text(), "qux bar qux baz qux");
}

#[test]
fn test_undo_and_redo_after_replace() {
    let mut engine = SearchEngine::default();
    let mut buffer = TextBuffer::from_str("original text here");

    engine.search(&buffer, "original").unwrap();
    engine.replace_current(&mut buffer, "modified").unwrap();
    assert_eq!(buffer.text(), "modified text here");

    // Undo restore (single transaction group)
    buffer.undo().unwrap();
    assert_eq!(buffer.text(), "original text here");

    // Redo re-apply
    buffer.redo().unwrap();
    assert_eq!(buffer.text(), "modified text here");
}

#[test]
fn test_search_history_and_cache() {
    let mut engine = SearchEngine::default();
    let buffer = TextBuffer::from_str("sample content for caching and history");

    engine.search(&buffer, "sample").unwrap();
    engine.search(&buffer, "content").unwrap();

    let history = engine.history();
    assert_eq!(history.queries(), &["content", "sample"]);

    // Statistics telemetry
    let stats = engine.statistics();
    assert_eq!(stats.total_matches, 1);
    assert!(stats.is_active);
}

#[test]
fn test_large_buffer_performance() {
    let mut engine = SearchEngine::default();
    let mut lines = Vec::with_capacity(10_000);
    for i in 0..10_000 {
        lines.push(format!("log entry line #{}: status = OK, code = 200", i));
    }
    let buffer = TextBuffer::from_lines(lines);

    let matches = engine.search(&buffer, "code = 200").unwrap();
    assert_eq!(matches.len(), 10_000);
    assert!(engine.statistics().search_duration_us > 0);
}
