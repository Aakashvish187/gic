use thiserror::Error;

/// Dedicated error enum for Search and Replace operations.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum SearchError {
    /// Search query string was empty.
    #[error("Search query cannot be empty")]
    EmptyQuery,

    /// Invalid regex or search pattern.
    #[error("Invalid search pattern: {0}")]
    InvalidPattern(String),

    /// Requested match was not found in buffer.
    #[error("Match not found")]
    MatchNotFound,

    /// Specified search range or index is out of bounds.
    #[error("Invalid search range or position: {0}")]
    InvalidRange(String),

    /// Replacement execution failed.
    #[error("Replace operation failed: {0}")]
    ReplaceFailed(String),

    /// Internal search buffer error.
    #[error("Buffer operational error: {0}")]
    BufferError(String),

    /// Operational search engine error.
    #[error("Search execution error: {0}")]
    SearchExecutionFailed(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_error_display() {
        assert_eq!(
            SearchError::EmptyQuery.to_string(),
            "Search query cannot be empty"
        );
        assert_eq!(
            SearchError::InvalidPattern("unbalanced [".to_string()).to_string(),
            "Invalid search pattern: unbalanced ["
        );
        assert_eq!(SearchError::MatchNotFound.to_string(), "Match not found");
    }
}
