use crate::search::matcher::MatchRange;
use serde::{Deserialize, Serialize};

/// Type or role of search match highlight.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HighlightKind {
    /// Active/selected current match.
    CurrentMatch,
    /// Inactive/secondary search match.
    OtherMatch,
}

/// Represents a visual highlight region for rendering components.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct HighlightRange {
    /// Text coordinates of the highlight.
    pub range: MatchRange,
    /// Highlight classification kind.
    pub kind: HighlightKind,
}

impl HighlightRange {
    /// Creates a new `HighlightRange`.
    pub fn new(range: MatchRange, kind: HighlightKind) -> Self {
        Self { range, kind }
    }
}

/// Computes optimized match highlights for rendering viewports.
#[derive(Debug, Clone, Default)]
pub struct HighlightEngine;

impl HighlightEngine {
    /// Creates a new `HighlightEngine`.
    pub fn new() -> Self {
        Self
    }

    /// Generates highlight ranges filtered by visible line viewport range `(start_row..=end_row)`.
    pub fn compute_highlights(
        &self,
        matches: &[crate::search::matcher::SearchMatch],
        current_match_index: Option<usize>,
        viewport_start_row: usize,
        viewport_end_row: usize,
        highlight_all: bool,
    ) -> Vec<HighlightRange> {
        let mut highlights = Vec::new();

        if matches.is_empty() {
            return highlights;
        }

        for (idx, m) in matches.iter().enumerate() {
            let row = m.range.row();

            // Skip matches outside visible viewport bounds
            if row < viewport_start_row || row > viewport_end_row {
                continue;
            }

            let is_current = current_match_index == Some(idx);

            if is_current {
                highlights.push(HighlightRange::new(m.range, HighlightKind::CurrentMatch));
            } else if highlight_all {
                highlights.push(HighlightRange::new(m.range, HighlightKind::OtherMatch));
            }
        }

        highlights
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::buffer::CursorPosition;
    use crate::search::matcher::SearchMatch;

    #[test]
    fn test_compute_viewport_highlights() {
        let engine = HighlightEngine::new();
        let m1 = SearchMatch::new(
            0,
            MatchRange::new(CursorPosition::new(1, 0), CursorPosition::new(1, 4)),
            "test".to_string(),
        );
        let m2 = SearchMatch::new(
            1,
            MatchRange::new(CursorPosition::new(5, 0), CursorPosition::new(5, 4)),
            "test".to_string(),
        );
        let m3 = SearchMatch::new(
            2,
            MatchRange::new(CursorPosition::new(20, 0), CursorPosition::new(20, 4)),
            "test".to_string(),
        );

        let matches = vec![m1, m2, m3];

        // Viewport rows 0..=10, current match index = 1 (m2)
        let hl = engine.compute_highlights(&matches, Some(1), 0, 10, true);
        assert_eq!(hl.len(), 2); // m1 and m2, m3 is out of viewport
        assert_eq!(hl[0].kind, HighlightKind::OtherMatch);
        assert_eq!(hl[1].kind, HighlightKind::CurrentMatch);
    }
}
