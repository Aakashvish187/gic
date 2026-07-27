/// Utilities for matching text against various patterns.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StringMatcher {
    /// Exact string match.
    Exact(String),
    /// Placeholder for a Regex match (avoids direct regex crate dependency in core for now).
    RegexPlaceholder(String),
    /// Substring match.
    Contains(String),
    /// Prefix match.
    StartsWith(String),
    /// Suffix match.
    EndsWith(String),
}

impl StringMatcher {
    /// Evaluates the matcher against the provided text.
    pub fn matches(&self, text: &str) -> bool {
        match self {
            Self::Exact(s) => text == s,
            Self::Contains(s) => text.contains(s),
            Self::StartsWith(s) => text.starts_with(s),
            Self::EndsWith(s) => text.ends_with(s),
            Self::RegexPlaceholder(_pattern) => {
                // In a full implementation, this would compile the regex and match.
                // For milestone 14, we just provide the structural placeholder.
                false
            }
        }
    }
}
