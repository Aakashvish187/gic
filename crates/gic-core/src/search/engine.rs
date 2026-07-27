use crate::buffer::{CursorPosition, TextBuffer};
use crate::search::cache::SearchCache;
use crate::search::errors::SearchError;
use crate::search::highlights::{HighlightEngine, HighlightRange};
use crate::search::history::SearchHistory;
use crate::search::matcher::{HorspoolMatcher, PatternMatcher, SearchMatch};
use crate::search::navigator::MatchNavigator;
use crate::search::options::SearchOptions;
use crate::search::query::SearchQuery;
use crate::search::replace::{ReplaceEngine, ReplaceResult};
use crate::search::statistics::SearchStatistics;
use std::time::Instant;

/// Central Search Engine coordinating pattern matching, navigation, highlighting, replacement, history, and caching.
#[derive(Debug)]
pub struct SearchEngine {
    options: SearchOptions,
    matcher: HorspoolMatcher,
    navigator: MatchNavigator,
    highlight_engine: HighlightEngine,
    replace_engine: ReplaceEngine,
    history: SearchHistory,
    cache: SearchCache,
    statistics: SearchStatistics,
    current_query: Option<SearchQuery>,
}

impl Default for SearchEngine {
    fn default() -> Self {
        Self::new(SearchOptions::default())
    }
}

impl SearchEngine {
    /// Creates a new `SearchEngine` with specified options.
    pub fn new(options: SearchOptions) -> Self {
        Self {
            options,
            matcher: HorspoolMatcher::new(),
            navigator: MatchNavigator::new(),
            highlight_engine: HighlightEngine::new(),
            replace_engine: ReplaceEngine::new(),
            history: SearchHistory::default(),
            cache: SearchCache::new(100),
            statistics: SearchStatistics::new(),
            current_query: None,
        }
    }

    // --- Search Execution ---

    /// Executes full search for `query_str` over `buffer`.
    pub fn search(
        &mut self,
        buffer: &TextBuffer,
        query_str: &str,
    ) -> Result<&[SearchMatch], SearchError> {
        let start_time = Instant::now();

        if query_str.is_empty() {
            self.clear();
            return Err(SearchError::EmptyQuery);
        }

        let query = SearchQuery::new(query_str, self.options.clone())?;
        self.history.add_query(query_str);

        let lines = buffer.lines();
        let buffer_hash = SearchCache::hash_buffer(lines);

        // Check search cache
        let matches = if let Some(cached) = self.cache.get(query_str, &self.options, buffer_hash) {
            cached.clone()
        } else {
            let computed = self.matcher.find_all(lines, &query);
            self.cache.put(
                query_str,
                self.options.clone(),
                buffer_hash,
                computed.clone(),
            );
            computed
        };

        let duration_us = start_time.elapsed().as_micros();

        // Update navigator and current query
        let cursor_pos = buffer.cursor_position();
        self.navigator.set_matches(matches, Some(cursor_pos));
        self.current_query = Some(query);

        // Update statistics
        self.statistics.total_matches = self.navigator.total_matches();
        self.statistics.current_match_index = self.navigator.current_index();
        self.statistics.search_duration_us = duration_us;
        self.statistics.current_query = Some(query_str.to_string());
        self.statistics.is_active = true;

        Ok(self.navigator.matches())
    }

    /// Performs non-blocking incremental search (updates matches as query string changes).
    pub fn incremental_search(
        &mut self,
        buffer: &TextBuffer,
        partial_query: &str,
    ) -> Result<&[SearchMatch], SearchError> {
        if partial_query.is_empty() {
            self.clear();
            return Ok(&[]);
        }

        self.search(buffer, partial_query)
    }

    // --- Navigation APIs ---

    /// Navigates to next match.
    pub fn next_match(&mut self) -> Option<&SearchMatch> {
        let wrap = self.options.wrap_around;
        let _ = self.navigator.next_match(wrap);
        self.statistics.current_match_index = self.navigator.current_index();
        self.navigator.current_match()
    }

    /// Navigates to previous match.
    pub fn previous_match(&mut self) -> Option<&SearchMatch> {
        let wrap = self.options.wrap_around;
        let _ = self.navigator.previous_match(wrap);
        self.statistics.current_match_index = self.navigator.current_index();
        self.navigator.current_match()
    }

    /// Jumps to specified match index.
    pub fn jump_to_match(&mut self, index: usize) -> Option<&SearchMatch> {
        let _ = self.navigator.jump_to(index);
        self.statistics.current_match_index = self.navigator.current_index();
        self.navigator.current_match()
    }

    /// Synchronizes active match index to cursor position.
    pub fn sync_cursor(&mut self, cursor_pos: CursorPosition) {
        self.navigator.sync_with_cursor(cursor_pos);
        self.statistics.current_match_index = self.navigator.current_index();
    }

    /// Returns active selected match.
    pub fn current_match(&self) -> Option<&SearchMatch> {
        self.navigator.current_match()
    }

    /// Returns total matches count.
    pub fn total_matches(&self) -> usize {
        self.navigator.total_matches()
    }

    // --- Highlight API ---

    /// Generates highlight ranges for specified visible line viewport `(start_row..=end_row)`.
    pub fn get_highlights(&self, start_row: usize, end_row: usize) -> Vec<HighlightRange> {
        self.highlight_engine.compute_highlights(
            self.navigator.matches(),
            self.navigator.current_index(),
            start_row,
            end_row,
            self.options.highlight_all,
        )
    }

    // --- Replace APIs ---

    /// Replaces active current match in `buffer` with `replacement` string.
    pub fn replace_current(
        &mut self,
        buffer: &mut TextBuffer,
        replacement: &str,
    ) -> Result<ReplaceResult, SearchError> {
        let current = self
            .navigator
            .current_match()
            .cloned()
            .ok_or(SearchError::MatchNotFound)?;

        self.history.add_replace(replacement);

        let res = self
            .replace_engine
            .replace_current(buffer, &current, replacement)?;

        self.cache.invalidate();
        self.statistics.replace_count += res.replacements_count;

        // Re-execute search to refresh matches after edit
        if let Some(query) = self.current_query.clone() {
            let _ = self.search(buffer, query.raw_query());
        }

        Ok(res)
    }

    /// Replaces all matches in `buffer` with `replacement` string.
    pub fn replace_all(
        &mut self,
        buffer: &mut TextBuffer,
        replacement: &str,
    ) -> Result<ReplaceResult, SearchError> {
        if self.navigator.total_matches() == 0 {
            return Ok(ReplaceResult::new(0, None));
        }

        self.history.add_replace(replacement);

        let matches = self.navigator.matches().to_vec();
        let res = self
            .replace_engine
            .replace_all(buffer, &matches, replacement)?;

        self.cache.invalidate();
        self.statistics.replace_count += res.replacements_count;

        // Re-execute search to refresh matches after bulk replace
        if let Some(query) = self.current_query.clone() {
            let _ = self.search(buffer, query.raw_query());
        }

        Ok(res)
    }

    // --- Configuration & Accessors ---

    /// Returns reference to search options.
    pub fn options(&self) -> &SearchOptions {
        &self.options
    }

    /// Returns mutable reference to search options.
    pub fn options_mut(&mut self) -> &mut SearchOptions {
        &mut self.options
    }

    /// Sets search options.
    pub fn set_options(&mut self, options: SearchOptions) {
        self.options = options;
        self.cache.invalidate();
    }

    /// Returns reference to search history.
    pub fn history(&self) -> &SearchHistory {
        &self.history
    }

    /// Returns mutable reference to search history.
    pub fn history_mut(&mut self) -> &mut SearchHistory {
        &mut self.history
    }

    /// Returns reference to search telemetry statistics.
    pub fn statistics(&self) -> &SearchStatistics {
        &self.statistics
    }

    /// Clears active search state, matches, and cache.
    pub fn clear(&mut self) {
        self.navigator.clear();
        self.cache.invalidate();
        self.statistics.reset();
        self.current_query = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_engine_flow() {
        let mut engine = SearchEngine::default();
        let buffer = TextBuffer::from_str("line 1: apple\nline 2: banana\nline 3: apple pie");

        let matches = engine.search(&buffer, "apple").unwrap();
        assert_eq!(matches.len(), 2);
        assert_eq!(engine.total_matches(), 2);

        // Next match navigation
        assert_eq!(
            engine.current_match().unwrap().range.start,
            CursorPosition::new(0, 8)
        );
        engine.next_match();
        assert_eq!(
            engine.current_match().unwrap().range.start,
            CursorPosition::new(2, 8)
        );

        // Highlights calculation
        let hl = engine.get_highlights(0, 5);
        assert_eq!(hl.len(), 2);
    }
}
