use crate::buffer::CursorPosition;
use crate::search::options::SearchMode;
use crate::search::query::SearchQuery;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Denotes start and end cursor positions of a match in the text buffer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct MatchRange {
    /// Starting position (row, col) inclusive.
    pub start: CursorPosition,
    /// Ending position (row, col) exclusive.
    pub end: CursorPosition,
}

impl MatchRange {
    /// Creates a new `MatchRange`.
    pub fn new(start: CursorPosition, end: CursorPosition) -> Self {
        Self { start, end }
    }

    /// Checks if a cursor position falls within this match range (start inclusive, end exclusive).
    pub fn contains(&self, pos: CursorPosition) -> bool {
        pos >= self.start && pos < self.end
    }

    /// Returns row index of match.
    pub fn row(&self) -> usize {
        self.start.row
    }
}

/// Represents an individual match result found within text.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SearchMatch {
    /// Unique match identifier index.
    pub id: usize,
    /// Match range coordinates.
    pub range: MatchRange,
    /// Exact text matched.
    pub matched_text: String,
}

impl SearchMatch {
    /// Creates a new `SearchMatch`.
    pub fn new(id: usize, range: MatchRange, matched_text: String) -> Self {
        Self {
            id,
            range,
            matched_text,
        }
    }
}

/// Trait defining pattern matching engine capabilities.
pub trait PatternMatcher: Send + Sync {
    /// Finds matches within a single line.
    fn find_matches_in_line(&self, line: &str, row: usize, query: &SearchQuery)
        -> Vec<SearchMatch>;

    /// Finds all matches across multiple lines.
    fn find_all(&self, lines: &[String], query: &SearchQuery) -> Vec<SearchMatch>;
}

/// Production implementation of Boyer-Moore-Horspool algorithm for UTF-8 line search.
#[derive(Debug, Clone, Default)]
pub struct HorspoolMatcher;

impl HorspoolMatcher {
    /// Creates a new `HorspoolMatcher`.
    pub fn new() -> Self {
        Self
    }

    /// Checks if character is a word constituent (alphanumeric or underscore).
    #[inline]
    fn is_word_char(c: char) -> bool {
        c.is_alphanumeric() || c == '_'
    }

    /// Constructs Horspool bad-character shift table for character scalars.
    fn build_shift_table(needle: &[char]) -> (HashMap<char, usize>, usize) {
        let len = needle.len();
        let mut table = HashMap::with_capacity(len);

        if len > 1 {
            for i in 0..(len - 1) {
                table.insert(needle[i], len - 1 - i);
            }
        }

        (table, len)
    }

    /// Validates word boundary constraints for whole word, prefix, or suffix matching modes.
    fn validate_boundary(
        chars: &[char],
        start_idx: usize,
        match_len: usize,
        mode: SearchMode,
    ) -> bool {
        let end_idx = start_idx + match_len;

        let char_before = if start_idx > 0 {
            chars.get(start_idx - 1).copied()
        } else {
            None
        };

        let char_after = chars.get(end_idx).copied();

        match mode {
            SearchMode::Literal | SearchMode::RegexPlaceholder => true,
            SearchMode::WholeWord => {
                let left_ok = match char_before {
                    Some(c) => !Self::is_word_char(c),
                    None => true,
                };
                let right_ok = match char_after {
                    Some(c) => !Self::is_word_char(c),
                    None => true,
                };
                left_ok && right_ok
            }
            SearchMode::Prefix => match char_before {
                Some(c) => !Self::is_word_char(c),
                None => true,
            },
            SearchMode::Suffix => match char_after {
                Some(c) => !Self::is_word_char(c),
                None => true,
            },
        }
    }
}

impl PatternMatcher for HorspoolMatcher {
    fn find_matches_in_line(
        &self,
        line: &str,
        row: usize,
        query: &SearchQuery,
    ) -> Vec<SearchMatch> {
        let mut matches = Vec::new();

        if line.is_empty() {
            return matches;
        }

        let is_case_sensitive = query.options().case_sensitive;
        let mode = query.options().mode;

        // Convert line and needle to character vectors for UTF-8 character indexing
        let line_chars: Vec<char> = if is_case_sensitive {
            line.chars().collect()
        } else {
            line.to_lowercase().chars().collect()
        };

        let raw_line_chars: Vec<char> = line.chars().collect();
        let needle_chars: Vec<char> = query.prepared_needle().chars().collect();

        let n = line_chars.len();
        let m = needle_chars.len();

        if m == 0 || m > n {
            return matches;
        }

        let (shift_table, default_shift) = Self::build_shift_table(&needle_chars);
        let mut idx = 0;

        while idx <= n - m {
            let mut j = (m - 1) as isize;

            while j >= 0 && line_chars[idx + j as usize] == needle_chars[j as usize] {
                j -= 1;
            }

            if j < 0 {
                // Match found! Verify word boundary conditions if applicable.
                if Self::validate_boundary(&raw_line_chars, idx, m, mode) {
                    let matched_str: String = raw_line_chars[idx..(idx + m)].iter().collect();
                    let start_pos = CursorPosition::new(row, idx);
                    let end_pos = CursorPosition::new(row, idx + m);
                    let range = MatchRange::new(start_pos, end_pos);

                    matches.push(SearchMatch::new(0, range, matched_str));
                }
                idx += 1; // Advance past match
            } else {
                let last_char = line_chars[idx + m - 1];
                let shift = shift_table
                    .get(&last_char)
                    .copied()
                    .unwrap_or(default_shift);
                idx += shift;
            }
        }

        matches
    }

    fn find_all(&self, lines: &[String], query: &SearchQuery) -> Vec<SearchMatch> {
        let mut all_matches = Vec::new();
        let mut match_counter = 0;

        for (row, line) in lines.iter().enumerate() {
            let line_matches = self.find_matches_in_line(line, row, query);
            for mut m in line_matches {
                m.id = match_counter;
                all_matches.push(m);
                match_counter += 1;
            }
        }

        all_matches
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::search::options::SearchOptions;

    #[test]
    fn test_horspool_literal_search() {
        let matcher = HorspoolMatcher::new();
        let opts = SearchOptions::default();
        let query = SearchQuery::new("test", opts).unwrap();
        let line = "this is a test line with test content";

        let matches = matcher.find_matches_in_line(line, 0, &query);
        assert_eq!(matches.len(), 2);
        assert_eq!(matches[0].range.start, CursorPosition::new(0, 10));
        assert_eq!(matches[0].range.end, CursorPosition::new(0, 14));
        assert_eq!(matches[1].range.start, CursorPosition::new(0, 25));
        assert_eq!(matches[1].range.end, CursorPosition::new(0, 29));
    }

    #[test]
    fn test_horspool_whole_word_search() {
        let matcher = HorspoolMatcher::new();
        let opts = SearchOptions::new().with_mode(SearchMode::WholeWord);
        let query = SearchQuery::new("cat", opts).unwrap();
        let line = "the cat concatenated with category cat";

        let matches = matcher.find_matches_in_line(line, 0, &query);
        assert_eq!(matches.len(), 2);
        assert_eq!(matches[0].range.start, CursorPosition::new(0, 4));
        assert_eq!(matches[1].range.start, CursorPosition::new(0, 35));
    }

    #[test]
    fn test_unicode_utf8_emoji_search() {
        let matcher = HorspoolMatcher::new();
        let opts = SearchOptions::default();
        let query = SearchQuery::new("🚀", opts).unwrap();
        let line = "start 🚀 rocket 🚀 launch";

        let matches = matcher.find_matches_in_line(line, 0, &query);
        assert_eq!(matches.len(), 2);
        assert_eq!(matches[0].range.start, CursorPosition::new(0, 6));
        assert_eq!(matches[0].range.end, CursorPosition::new(0, 7));
        assert_eq!(matches[1].range.start, CursorPosition::new(0, 15));
        assert_eq!(matches[1].range.end, CursorPosition::new(0, 16));
    }
}
