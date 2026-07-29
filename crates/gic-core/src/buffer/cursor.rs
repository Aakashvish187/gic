use serde::{Deserialize, Serialize};

/// 0-indexed position within a text buffer (row, character column).
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default,
)]
pub struct CursorPosition {
    /// 0-indexed line index.
    pub row: usize,
    /// 0-indexed character scalar index within the line.
    pub col: usize,
}

impl CursorPosition {
    /// Creates a new `CursorPosition`.
    pub fn new(row: usize, col: usize) -> Self {
        Self { row, col }
    }

    /// Origin position (0, 0).
    pub fn zero() -> Self {
        Self { row: 0, col: 0 }
    }
}

/// Represents the active editing cursor with navigation and bounds-clamping logic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Cursor {
    /// Current position of the cursor.
    pub position: CursorPosition,
    /// Preferred column maintained when navigating vertically across variable-length lines.
    pub preferred_col: Option<usize>,
}

impl Default for Cursor {
    fn default() -> Self {
        Self::new()
    }
}

impl Cursor {
    /// Creates a new `Cursor` initialized at (0, 0).
    pub fn new() -> Self {
        Self {
            position: CursorPosition::zero(),
            preferred_col: None,
        }
    }

    /// Creates a `Cursor` initialized at specified row and col.
    pub fn at(row: usize, col: usize) -> Self {
        Self {
            position: CursorPosition::new(row, col),
            preferred_col: Some(col),
        }
    }

    /// Returns row index.
    pub fn row(&self) -> usize {
        self.position.row
    }

    /// Returns character column index.
    pub fn col(&self) -> usize {
        self.position.col
    }

    /// Moves cursor left by one character. Wraps to end of previous line if at col 0.
    pub fn move_left(&mut self, lines: &[String]) {
        self.preferred_col = None;
        if self.position.col > 0 {
            self.position.col -= 1;
        } else if self.position.row > 0 {
            self.position.row -= 1;
            self.position.col = get_line_char_count(lines, self.position.row);
        }
    }

    /// Moves cursor right by one character. Wraps to start of next line if at end of line.
    pub fn move_right(&mut self, lines: &[String]) {
        self.preferred_col = None;
        let char_count = get_line_char_count(lines, self.position.row);

        if self.position.col < char_count {
            self.position.col += 1;
        } else if self.position.row + 1 < lines.len() {
            self.position.row += 1;
            self.position.col = 0;
        }
    }

    /// Moves cursor up by one line, attempting to preserve preferred column.
    pub fn move_up(&mut self, lines: &[String]) {
        if self.position.row > 0 {
            let target_col = self.preferred_col.unwrap_or(self.position.col);
            self.preferred_col = Some(target_col);
            self.position.row -= 1;
            let line_len = get_line_char_count(lines, self.position.row);
            self.position.col = target_col.min(line_len);
        }
    }

    /// Moves cursor down by one line, attempting to preserve preferred column.
    pub fn move_down(&mut self, lines: &[String]) {
        if self.position.row + 1 < lines.len() {
            let target_col = self.preferred_col.unwrap_or(self.position.col);
            self.preferred_col = Some(target_col);
            self.position.row += 1;
            let line_len = get_line_char_count(lines, self.position.row);
            self.position.col = target_col.min(line_len);
        }
    }

    /// Moves cursor to start of current line (col 0).
    pub fn move_to_line_start(&mut self) {
        self.preferred_col = None;
        self.position.col = 0;
    }

    /// Moves cursor to end of current line.
    pub fn move_to_line_end(&mut self, lines: &[String]) {
        self.preferred_col = None;
        self.position.col = get_line_char_count(lines, self.position.row);
    }

    /// Moves cursor to top-left of file (0, 0).
    pub fn move_to_file_start(&mut self) {
        self.preferred_col = None;
        self.position = CursorPosition::zero();
    }

    /// Moves cursor to end of file (last line, end of last line).
    pub fn move_to_file_end(&mut self, lines: &[String]) {
        self.preferred_col = None;
        if lines.is_empty() {
            self.position = CursorPosition::zero();
        } else {
            let last_row = lines.len() - 1;
            let last_col = get_line_char_count(lines, last_row);
            self.position = CursorPosition::new(last_row, last_col);
        }
    }

    /// Moves cursor forward by one word boundary.
    pub fn move_word_forward(&mut self, lines: &[String]) {
        self.preferred_col = None;
        if lines.is_empty() {
            return;
        }

        let line = &lines[self.position.row];
        let chars: Vec<char> = line.chars().collect();
        let len = chars.len();

        if self.position.col >= len {
            if self.position.row + 1 < lines.len() {
                self.position.row += 1;
                self.position.col = 0;
            }
            return;
        }

        let mut idx = self.position.col;
        // Skip current word characters
        let initial_is_alphanumeric = chars[idx].is_alphanumeric();
        while idx < len
            && chars[idx].is_alphanumeric() == initial_is_alphanumeric
            && !chars[idx].is_whitespace()
        {
            idx += 1;
        }

        // Skip whitespace
        while idx < len && chars[idx].is_whitespace() {
            idx += 1;
        }

        if idx >= len && self.position.row + 1 < lines.len() {
            self.position.row += 1;
            self.position.col = 0;
        } else {
            self.position.col = idx;
        }
    }

    /// Moves cursor backward by one word boundary.
    pub fn move_word_backward(&mut self, lines: &[String]) {
        self.preferred_col = None;
        if lines.is_empty() {
            return;
        }

        if self.position.col == 0 {
            if self.position.row > 0 {
                self.position.row -= 1;
                self.position.col = get_line_char_count(lines, self.position.row);
            }
            return;
        }

        let line = &lines[self.position.row];
        let chars: Vec<char> = line.chars().collect();
        let mut idx = self.position.col;

        // Skip trailing whitespace
        while idx > 0 && chars[idx - 1].is_whitespace() {
            idx -= 1;
        }

        if idx == 0 {
            self.position.col = 0;
            return;
        }

        let initial_is_alphanumeric = chars[idx - 1].is_alphanumeric();
        while idx > 0
            && chars[idx - 1].is_alphanumeric() == initial_is_alphanumeric
            && !chars[idx - 1].is_whitespace()
        {
            idx -= 1;
        }

        self.position.col = idx;
    }

    /// Clamps cursor position to ensure row and column remain within valid buffer bounds.
    pub fn clamp(&mut self, lines: &[String]) {
        if lines.is_empty() {
            self.position = CursorPosition::zero();
            return;
        }

        if self.position.row >= lines.len() {
            self.position.row = lines.len() - 1;
        }

        let char_count = get_line_char_count(lines, self.position.row);
        if self.position.col > char_count {
            self.position.col = char_count;
        }
    }
}

/// Helper to get Unicode character scalar count for a given line.
fn get_line_char_count(lines: &[String], row: usize) -> usize {
    lines.get(row).map(|l| l.chars().count()).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_lines() -> Vec<String> {
        vec![
            "Hello World".into(),       // len = 11
            "Short".into(),             // len = 5
            "A long third line".into(), // len = 17
        ]
    }

    #[test]
    fn test_cursor_left_right_basic() {
        let lines = test_lines();
        let mut cursor = Cursor::new();

        cursor.move_right(&lines);
        assert_eq!(cursor.col(), 1);

        cursor.move_left(&lines);
        assert_eq!(cursor.col(), 0);

        // Move left at start of line -> wraps to end of previous line (none here, stays 0)
        cursor.move_left(&lines);
        assert_eq!(cursor.position, CursorPosition::zero());
    }

    #[test]
    fn test_cursor_line_wrap() {
        let lines = test_lines();
        let mut cursor = Cursor::at(0, 11);

        // Right at line end -> wraps to start of line 1
        cursor.move_right(&lines);
        assert_eq!(cursor.position, CursorPosition::new(1, 0));

        // Left at start of line 1 -> wraps to end of line 0
        cursor.move_left(&lines);
        assert_eq!(cursor.position, CursorPosition::new(0, 11));
    }

    #[test]
    fn test_cursor_up_down_preferred_col() {
        let lines = test_lines();
        let mut cursor = Cursor::at(0, 10);

        // Down to line 1 (len 5) -> col clamped to 5
        cursor.move_down(&lines);
        assert_eq!(cursor.position, CursorPosition::new(1, 5));

        // Down to line 2 (len 17) -> col restored to preferred 10
        cursor.move_down(&lines);
        assert_eq!(cursor.position, CursorPosition::new(2, 10));

        // Up to line 1 -> col clamped to 5
        cursor.move_up(&lines);
        assert_eq!(cursor.position, CursorPosition::new(1, 5));
    }

    #[test]
    fn test_cursor_word_movement() {
        let lines = vec!["hello world  foo".into()];
        let mut cursor = Cursor::new();

        cursor.move_word_forward(&lines);
        assert_eq!(cursor.col(), 6); // start of "world"

        cursor.move_word_forward(&lines);
        assert_eq!(cursor.col(), 13); // start of "foo"

        cursor.move_word_backward(&lines);
        assert_eq!(cursor.col(), 6); // start of "world"

        cursor.move_word_backward(&lines);
        assert_eq!(cursor.col(), 0); // start of "hello"
    }

    #[test]
    fn test_cursor_clamping() {
        let lines = test_lines();
        let mut cursor = Cursor::at(10, 100);

        cursor.clamp(&lines);
        assert_eq!(cursor.row(), 2);
        assert_eq!(cursor.col(), 17);
    }
}
