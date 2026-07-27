use crate::buffer::{CursorPosition, TextBuffer};
use crate::search::errors::SearchError;
use crate::search::matcher::SearchMatch;
use serde::{Deserialize, Serialize};

/// Summary result of replacement operations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplaceResult {
    /// Number of successful replacements executed.
    pub replacements_count: usize,
    /// Updated cursor position post-replacement if applicable.
    pub new_cursor_position: Option<CursorPosition>,
}

impl ReplaceResult {
    /// Creates a new `ReplaceResult`.
    pub fn new(replacements_count: usize, new_cursor_position: Option<CursorPosition>) -> Self {
        Self {
            replacements_count,
            new_cursor_position,
        }
    }
}

/// Executes single and bulk replace operations against a `TextBuffer`.
#[derive(Debug, Clone, Default)]
pub struct ReplaceEngine;

impl ReplaceEngine {
    /// Creates a new `ReplaceEngine`.
    pub fn new() -> Self {
        Self
    }

    /// Replaces a single `SearchMatch` within the text buffer with `replacement` string.
    pub fn replace_current(
        &self,
        buffer: &mut TextBuffer,
        match_info: &SearchMatch,
        replacement: &str,
    ) -> Result<ReplaceResult, SearchError> {
        let start = match_info.range.start;
        let end = match_info.range.end;

        // Perform unified replace_range on TextBuffer
        buffer
            .replace_range(start, end, replacement)
            .map_err(|e| SearchError::ReplaceFailed(e.to_string()))?;

        let new_cursor_pos = buffer.cursor_position();

        Ok(ReplaceResult::new(1, Some(new_cursor_pos)))
    }

    /// Replaces all matches in `matches` slice with `replacement` string.
    /// Processes matches in reverse order (bottom-to-top, right-to-left) to preserve coordinate validity.
    pub fn replace_all(
        &self,
        buffer: &mut TextBuffer,
        matches: &[SearchMatch],
        replacement: &str,
    ) -> Result<ReplaceResult, SearchError> {
        if matches.is_empty() {
            return Ok(ReplaceResult::new(0, None));
        }

        let mut count = 0;
        let mut last_pos = None;

        // Sort / process matches in reverse order by start position
        let mut sorted_matches = matches.to_vec();
        sorted_matches.sort_by_key(|b| std::cmp::Reverse(b.range.start));

        for m in sorted_matches {
            let start = m.range.start;
            let end = m.range.end;

            buffer
                .replace_range(start, end, replacement)
                .map_err(|e| SearchError::ReplaceFailed(e.to_string()))?;

            last_pos = Some(buffer.cursor_position());
            count += 1;
        }

        Ok(ReplaceResult::new(count, last_pos))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::search::matcher::MatchRange;

    #[test]
    fn test_replace_current() {
        let mut buffer = TextBuffer::from_str("hello world line");
        let engine = ReplaceEngine::new();

        let m = SearchMatch::new(
            0,
            MatchRange::new(CursorPosition::new(0, 6), CursorPosition::new(0, 11)),
            "world".to_string(),
        );

        let res = engine.replace_current(&mut buffer, &m, "rust").unwrap();
        assert_eq!(res.replacements_count, 1);
        assert_eq!(buffer.text(), "hello rust line");
        assert_eq!(res.new_cursor_position, Some(CursorPosition::new(0, 10)));
    }

    #[test]
    fn test_replace_all() {
        let mut buffer = TextBuffer::from_str("cat cat cat");
        let engine = ReplaceEngine::new();

        let m1 = SearchMatch::new(
            0,
            MatchRange::new(CursorPosition::new(0, 0), CursorPosition::new(0, 3)),
            "cat".to_string(),
        );
        let m2 = SearchMatch::new(
            1,
            MatchRange::new(CursorPosition::new(0, 4), CursorPosition::new(0, 7)),
            "cat".to_string(),
        );
        let m3 = SearchMatch::new(
            2,
            MatchRange::new(CursorPosition::new(0, 8), CursorPosition::new(0, 11)),
            "cat".to_string(),
        );

        let res = engine
            .replace_all(&mut buffer, &[m1, m2, m3], "dog")
            .unwrap();
        assert_eq!(res.replacements_count, 3);
        assert_eq!(buffer.text(), "dog dog dog");
    }
}
