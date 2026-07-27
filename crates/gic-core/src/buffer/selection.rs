use crate::buffer::cursor::CursorPosition;
use serde::{Deserialize, Serialize};

/// Selection highlight mode in the text buffer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[derive(Default)]
pub enum SelectionMode {
    /// Standard continuous character selection.
    #[default]
    Character,
    /// Full line selection mode.
    Line,
    /// Rectangular block selection mode.
    Block,
}


/// Represents an active or inactive text selection range.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Selection {
    /// Fixed starting anchor position of the selection.
    pub anchor: CursorPosition,
    /// Current dynamic head position of the selection.
    pub head: CursorPosition,
    /// Mode of selection (Character, Line, Block).
    pub mode: SelectionMode,
    /// True if an active selection is enabled.
    pub is_active: bool,
}

impl Default for Selection {
    fn default() -> Self {
        Self::new()
    }
}

impl Selection {
    /// Creates a new inactive `Selection`.
    pub fn new() -> Self {
        Self {
            anchor: CursorPosition::zero(),
            head: CursorPosition::zero(),
            mode: SelectionMode::Character,
            is_active: false,
        }
    }

    /// Starts a selection at specified anchor position and mode.
    pub fn start(&mut self, anchor: CursorPosition, mode: SelectionMode) {
        self.anchor = anchor;
        self.head = anchor;
        self.mode = mode;
        self.is_active = true;
    }

    /// Updates the dynamic head of the selection.
    pub fn update(&mut self, head: CursorPosition) {
        if self.is_active {
            self.head = head;
        }
    }

    /// Clears and deactivates current selection.
    pub fn clear(&mut self) {
        self.is_active = false;
        self.anchor = CursorPosition::zero();
        self.head = CursorPosition::zero();
    }

    /// Returns normalized `(start_pos, end_pos)` such that `start_pos <= end_pos`.
    pub fn range(&self) -> (CursorPosition, CursorPosition) {
        if !self.is_active {
            return (self.anchor, self.head);
        }

        match self.mode {
            SelectionMode::Character => {
                if self.anchor <= self.head {
                    (self.anchor, self.head)
                } else {
                    (self.head, self.anchor)
                }
            }
            SelectionMode::Line => {
                let start_row = self.anchor.row.min(self.head.row);
                let end_row = self.anchor.row.max(self.head.row);
                (
                    CursorPosition::new(start_row, 0),
                    CursorPosition::new(end_row, usize::MAX),
                )
            }
            SelectionMode::Block => {
                let start_row = self.anchor.row.min(self.head.row);
                let end_row = self.anchor.row.max(self.head.row);
                let start_col = self.anchor.col.min(self.head.col);
                let end_col = self.anchor.col.max(self.head.col);
                (
                    CursorPosition::new(start_row, start_col),
                    CursorPosition::new(end_row, end_col),
                )
            }
        }
    }

    /// Returns true if given `pos` falls within active selection bounds.
    pub fn contains(&self, pos: CursorPosition) -> bool {
        if !self.is_active {
            return false;
        }
        let (start, end) = self.range();
        pos >= start && pos <= end
    }

    /// Extracts the selected text from given lines vector.
    pub fn get_selected_text(&self, lines: &[String]) -> Option<String> {
        if !self.is_active || lines.is_empty() {
            return None;
        }

        let (start, end) = self.range();
        let start_row = start.row.min(lines.len() - 1);
        let end_row = end.row.min(lines.len() - 1);

        if start_row == end_row {
            let line = &lines[start_row];
            let chars: Vec<char> = line.chars().collect();
            let start_col = start.col.min(chars.len());
            let end_col = end.col.min(chars.len());

            if start_col >= end_col {
                return Some(String::new());
            }
            let selected_slice: String = chars[start_col..end_col].iter().collect();
            return Some(selected_slice);
        }

        let mut result = Vec::new();
        for r in start_row..=end_row {
            let line = &lines[r];
            let chars: Vec<char> = line.chars().collect();

            if r == start_row {
                let start_col = start.col.min(chars.len());
                let slice: String = chars[start_col..].iter().collect();
                result.push(slice);
            } else if r == end_row {
                let end_col = end.col.min(chars.len());
                let slice: String = chars[..end_col].iter().collect();
                result.push(slice);
            } else {
                result.push(line.clone());
            }
        }

        Some(result.join("\n"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_selection_start_update_clear() {
        let mut selection = Selection::new();
        assert!(!selection.is_active);

        selection.start(CursorPosition::new(0, 5), SelectionMode::Character);
        assert!(selection.is_active);
        assert_eq!(selection.anchor, CursorPosition::new(0, 5));

        selection.update(CursorPosition::new(2, 10));
        assert_eq!(selection.head, CursorPosition::new(2, 10));

        let (start, end) = selection.range();
        assert_eq!(start, CursorPosition::new(0, 5));
        assert_eq!(end, CursorPosition::new(2, 10));

        selection.clear();
        assert!(!selection.is_active);
    }

    #[test]
    fn test_selection_text_extraction() {
        let lines = vec![
            "First Line".into(),  // row 0
            "Second Line".into(), // row 1
            "Third Line".into(),  // row 2
        ];

        let mut selection = Selection::new();
        selection.start(CursorPosition::new(0, 6), SelectionMode::Character);
        selection.update(CursorPosition::new(1, 6));

        let extracted = selection.get_selected_text(&lines).unwrap();
        assert_eq!(extracted, "Line\nSecond");
    }
}
