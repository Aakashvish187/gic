use crate::buffer::CursorPosition;
use crate::search::matcher::SearchMatch;
use serde::{Deserialize, Serialize};

/// Manages active search match traversal, navigation indices, and cursor synchronization.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct MatchNavigator {
    matches: Vec<SearchMatch>,
    current_index: Option<usize>,
}

impl MatchNavigator {
    /// Creates a new empty `MatchNavigator`.
    pub fn new() -> Self {
        Self {
            matches: Vec::new(),
            current_index: None,
        }
    }

    /// Sets/replaces the active match list and resets or preserves navigation position.
    pub fn set_matches(&mut self, matches: Vec<SearchMatch>, cursor_pos: Option<CursorPosition>) {
        self.matches = matches;
        if self.matches.is_empty() {
            self.current_index = None;
        } else if let Some(pos) = cursor_pos {
            self.sync_with_cursor(pos);
        } else {
            self.current_index = Some(0);
        }
    }

    /// Clears all matches.
    pub fn clear(&mut self) {
        self.matches.clear();
        self.current_index = None;
    }

    /// Returns total match count.
    pub fn total_matches(&self) -> usize {
        self.matches.len()
    }

    /// Returns 0-indexed current match index if matches exist.
    pub fn current_index(&self) -> Option<usize> {
        self.current_index
    }

    /// Returns reference to current `SearchMatch` if selected.
    pub fn current_match(&self) -> Option<&SearchMatch> {
        self.current_index.and_then(|idx| self.matches.get(idx))
    }

    /// Returns slice of all matches.
    pub fn matches(&self) -> &[SearchMatch] {
        &self.matches
    }

    /// Synchronizes current match index to the match at or immediately following `cursor_pos`.
    pub fn sync_with_cursor(&mut self, cursor_pos: CursorPosition) {
        if self.matches.is_empty() {
            self.current_index = None;
            return;
        }

        // Find first match whose start position >= cursor_pos
        let idx = self
            .matches
            .iter()
            .position(|m| m.range.start >= cursor_pos)
            .unwrap_or(0); // If past all matches, point to first match

        self.current_index = Some(idx);
    }

    /// Navigates to next match. If `wrap_around` is true, wraps to top upon reaching end.
    pub fn next_match(&mut self, wrap_around: bool) -> Option<&SearchMatch> {
        if self.matches.is_empty() {
            return None;
        }

        let next_idx = match self.current_index {
            Some(curr) => {
                if curr + 1 < self.matches.len() {
                    curr + 1
                } else if wrap_around {
                    0
                } else {
                    curr
                }
            }
            None => 0,
        };

        self.current_index = Some(next_idx);
        self.matches.get(next_idx)
    }

    /// Navigates to previous match. If `wrap_around` is true, wraps to bottom upon reaching top.
    pub fn previous_match(&mut self, wrap_around: bool) -> Option<&SearchMatch> {
        if self.matches.is_empty() {
            return None;
        }

        let prev_idx = match self.current_index {
            Some(curr) => {
                if curr > 0 {
                    curr - 1
                } else if wrap_around {
                    self.matches.len() - 1
                } else {
                    0
                }
            }
            None => self.matches.len() - 1,
        };

        self.current_index = Some(prev_idx);
        self.matches.get(prev_idx)
    }

    /// Jumps directly to specified match index.
    pub fn jump_to(&mut self, index: usize) -> Option<&SearchMatch> {
        if index < self.matches.len() {
            self.current_index = Some(index);
            self.matches.get(index)
        } else {
            None
        }
    }

    /// Navigates to first match.
    pub fn first_match(&mut self) -> Option<&SearchMatch> {
        self.jump_to(0)
    }

    /// Navigates to last match.
    pub fn last_match(&mut self) -> Option<&SearchMatch> {
        if self.matches.is_empty() {
            None
        } else {
            self.jump_to(self.matches.len() - 1)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::search::matcher::MatchRange;

    fn make_dummy_match(id: usize, row: usize, col: usize) -> SearchMatch {
        let start = CursorPosition::new(row, col);
        let end = CursorPosition::new(row, col + 4);
        SearchMatch::new(id, MatchRange::new(start, end), "test".to_string())
    }

    #[test]
    fn test_navigator_navigation() {
        let mut nav = MatchNavigator::new();
        let m1 = make_dummy_match(0, 0, 10);
        let m2 = make_dummy_match(1, 2, 5);
        let m3 = make_dummy_match(2, 5, 0);

        nav.set_matches(vec![m1.clone(), m2.clone(), m3.clone()], None);
        assert_eq!(nav.total_matches(), 3);
        assert_eq!(nav.current_index(), Some(0));

        // Next match
        assert_eq!(nav.next_match(true).unwrap().id, 1);
        assert_eq!(nav.next_match(true).unwrap().id, 2);

        // Wrap around to 0
        assert_eq!(nav.next_match(true).unwrap().id, 0);

        // Previous match with wrap around
        assert_eq!(nav.previous_match(true).unwrap().id, 2);
    }

    #[test]
    fn test_sync_with_cursor() {
        let mut nav = MatchNavigator::new();
        let m1 = make_dummy_match(0, 0, 10);
        let m2 = make_dummy_match(1, 2, 5);
        let m3 = make_dummy_match(2, 5, 0);

        nav.set_matches(vec![m1, m2, m3], None);

        // Cursor at row 2, col 0 -> should sync to match at row 2, col 5 (index 1)
        nav.sync_with_cursor(CursorPosition::new(2, 0));
        assert_eq!(nav.current_index(), Some(1));
    }
}
