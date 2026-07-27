use crate::search::errors::SearchError;
use crate::search::matcher::SearchMatch;

/// Trait abstraction defining future regular expression pattern matching integration.
pub trait RegexEngine: Send + Sync {
    /// Compiles a regex pattern string into an executable matcher.
    fn compile(&self, pattern: &str, case_sensitive: bool) -> Result<(), SearchError>;

    /// Executes regex search against a single line string.
    fn find_line_matches(
        &self,
        line: &str,
        row: usize,
        pattern: &str,
    ) -> Result<Vec<SearchMatch>, SearchError>;
}

/// Placeholder implementation for future Regex engine integration.
#[derive(Debug, Clone, Default)]
pub struct PlaceholderRegexEngine;

impl PlaceholderRegexEngine {
    /// Creates a new `PlaceholderRegexEngine`.
    pub fn new() -> Self {
        Self
    }
}

impl RegexEngine for PlaceholderRegexEngine {
    fn compile(&self, _pattern: &str, _case_sensitive: bool) -> Result<(), SearchError> {
        Err(SearchError::InvalidPattern(
            "Regex search plugin engine not enabled in V1 engine".to_string(),
        ))
    }

    fn find_line_matches(
        &self,
        _line: &str,
        _row: usize,
        _pattern: &str,
    ) -> Result<Vec<SearchMatch>, SearchError> {
        Err(SearchError::InvalidPattern(
            "Regex search plugin engine not enabled in V1 engine".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_regex_placeholder() {
        let engine = PlaceholderRegexEngine::new();
        assert!(engine.compile(".*", true).is_err());
    }
}
