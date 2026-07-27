use crate::search::errors::SearchError;
use crate::search::options::SearchOptions;
use serde::{Deserialize, Serialize};

/// Represents a validated, prepared search query with precomputed properties.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SearchQuery {
    raw_query: String,
    prepared_needle: String,
    char_len: usize,
    options: SearchOptions,
}

impl SearchQuery {
    /// Creates and validates a new `SearchQuery`. Returns `SearchError::EmptyQuery` if input string is empty.
    pub fn new(query: &str, options: SearchOptions) -> Result<Self, SearchError> {
        if query.is_empty() {
            return Err(SearchError::EmptyQuery);
        }

        let prepared_needle = if options.case_sensitive {
            query.to_string()
        } else {
            query.to_lowercase()
        };

        let char_len = query.chars().count();

        Ok(Self {
            raw_query: query.to_string(),
            prepared_needle,
            char_len,
            options,
        })
    }

    /// Returns reference to original query text.
    pub fn raw_query(&self) -> &str {
        &self.raw_query
    }

    /// Returns reference to prepared search needle (lowercase if case-insensitive).
    pub fn prepared_needle(&self) -> &str {
        &self.prepared_needle
    }

    /// Returns character scalar count of search needle.
    pub fn char_len(&self) -> usize {
        self.char_len
    }

    /// Returns reference to search options.
    pub fn options(&self) -> &SearchOptions {
        &self.options
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_query() {
        let opts = SearchOptions::default();
        let query = SearchQuery::new("Hello", opts.clone()).unwrap();
        assert_eq!(query.raw_query(), "Hello");
        assert_eq!(query.prepared_needle(), "hello"); // default case_sensitive is false
        assert_eq!(query.char_len(), 5);
    }

    #[test]
    fn test_case_sensitive_query() {
        let opts = SearchOptions::new().with_case_sensitive(true);
        let query = SearchQuery::new("Hello", opts).unwrap();
        assert_eq!(query.prepared_needle(), "Hello");
    }

    #[test]
    fn test_empty_query_err() {
        let opts = SearchOptions::default();
        assert_eq!(SearchQuery::new("", opts), Err(SearchError::EmptyQuery));
    }
}
