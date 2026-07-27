use serde::{Deserialize, Serialize};

/// Mode of search pattern interpretation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[derive(Default)]
pub enum SearchMode {
    /// Exact literal text matching (default).
    #[default]
    Literal,
    /// Matches complete word boundaries (non-alphanumeric/underscore delimiters).
    WholeWord,
    /// Matches pattern only at line or word prefixes.
    Prefix,
    /// Matches pattern only at line or word suffixes.
    Suffix,
    /// Future regular expression matching support.
    RegexPlaceholder,
}


/// Search direction across the buffer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[derive(Default)]
pub enum SearchDirection {
    /// Search forward from cursor towards end of buffer.
    #[default]
    Forward,
    /// Search backward from cursor towards start of buffer.
    Backward,
}


/// Boundaries or scope for search execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[derive(Default)]
pub enum SearchScope {
    /// Search entire text buffer (default).
    #[default]
    FullBuffer,
    /// Search within current selection only.
    Selection,
    /// Search within active line only.
    CurrentLine,
    /// Search within visible viewport range `(start_row..=end_row)`.
    VisibleViewport { start_row: usize, end_row: usize },
}


/// Comprehensive search parameters and user preferences.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SearchOptions {
    /// True for case-sensitive match; false for case-insensitive.
    pub case_sensitive: bool,
    /// Pattern matching mode (Literal, WholeWord, Prefix, Suffix, etc.).
    pub mode: SearchMode,
    /// Wrap search to top/bottom when reaching boundary.
    pub wrap_around: bool,
    /// Highlight all occurrences in active viewport/buffer.
    pub highlight_all: bool,
    /// Direction of search traversal (Forward or Backward).
    pub direction: SearchDirection,
    /// Scope limit for search execution.
    pub scope: SearchScope,
    /// Future flag to preserve matching case during replace operations.
    pub preserve_case: bool,
}

impl Default for SearchOptions {
    fn default() -> Self {
        Self {
            case_sensitive: false,
            mode: SearchMode::Literal,
            wrap_around: true,
            highlight_all: true,
            direction: SearchDirection::Forward,
            scope: SearchScope::FullBuffer,
            preserve_case: false,
        }
    }
}

impl SearchOptions {
    /// Creates a default `SearchOptions` configuration.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets case sensitivity option.
    pub fn with_case_sensitive(mut self, case_sensitive: bool) -> Self {
        self.case_sensitive = case_sensitive;
        self
    }

    /// Sets matching mode.
    pub fn with_mode(mut self, mode: SearchMode) -> Self {
        self.mode = mode;
        self
    }

    /// Sets wrap around toggle.
    pub fn with_wrap_around(mut self, wrap_around: bool) -> Self {
        self.wrap_around = wrap_around;
        self
    }

    /// Sets highlight all toggle.
    pub fn with_highlight_all(mut self, highlight_all: bool) -> Self {
        self.highlight_all = highlight_all;
        self
    }

    /// Sets search direction.
    pub fn with_direction(mut self, direction: SearchDirection) -> Self {
        self.direction = direction;
        self
    }

    /// Sets search scope limit.
    pub fn with_scope(mut self, scope: SearchScope) -> Self {
        self.scope = scope;
        self
    }

    /// Sets preserve case toggle.
    pub fn with_preserve_case(mut self, preserve_case: bool) -> Self {
        self.preserve_case = preserve_case;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_options() {
        let opts = SearchOptions::default();
        assert!(!opts.case_sensitive);
        assert_eq!(opts.mode, SearchMode::Literal);
        assert!(opts.wrap_around);
        assert!(opts.highlight_all);
        assert_eq!(opts.direction, SearchDirection::Forward);
        assert_eq!(opts.scope, SearchScope::FullBuffer);
    }

    #[test]
    fn test_builder_pattern() {
        let opts = SearchOptions::new()
            .with_case_sensitive(true)
            .with_mode(SearchMode::WholeWord)
            .with_direction(SearchDirection::Backward);

        assert!(opts.case_sensitive);
        assert_eq!(opts.mode, SearchMode::WholeWord);
        assert_eq!(opts.direction, SearchDirection::Backward);
    }
}
