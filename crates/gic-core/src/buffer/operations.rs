use crate::buffer::cursor::CursorPosition;
use crate::buffer::errors::BufferError;

/// Core line storage mutation operations.
pub struct BufferOperations;

impl BufferOperations {
    /// Inserts a character into lines vector at given cursor position.
    pub fn insert_char(
        lines: &mut Vec<String>,
        pos: CursorPosition,
        ch: char,
    ) -> Result<(), BufferError> {
        if lines.is_empty() {
            lines.push(String::new());
        }

        if pos.row >= lines.len() {
            return Err(BufferError::InvalidPosition {
                row: pos.row,
                col: pos.col,
            });
        }

        if ch == '\n' {
            Self::split_line(lines, pos)?;
            return Ok(());
        }

        let line = &mut lines[pos.row];
        let mut chars: Vec<char> = line.chars().collect();

        if pos.col > chars.len() {
            return Err(BufferError::InvalidPosition {
                row: pos.row,
                col: pos.col,
            });
        }

        chars.insert(pos.col, ch);
        lines[pos.row] = chars.into_iter().collect();
        Ok(())
    }

    /// Inserts a string into lines vector at given cursor position.
    /// Returns the new resulting cursor position.
    pub fn insert_str(
        lines: &mut Vec<String>,
        pos: CursorPosition,
        text: &str,
    ) -> Result<CursorPosition, BufferError> {
        if lines.is_empty() {
            lines.push(String::new());
        }

        if pos.row >= lines.len() {
            return Err(BufferError::InvalidPosition {
                row: pos.row,
                col: pos.col,
            });
        }

        let input_lines: Vec<&str> = text.split('\n').collect();
        if input_lines.is_empty() {
            return Ok(pos);
        }

        if input_lines.len() == 1 {
            let line = &mut lines[pos.row];
            let mut chars: Vec<char> = line.chars().collect();
            if pos.col > chars.len() {
                return Err(BufferError::InvalidPosition {
                    row: pos.row,
                    col: pos.col,
                });
            }

            let inserted_chars: Vec<char> = input_lines[0].chars().collect();
            let added_count = inserted_chars.len();
            chars.splice(pos.col..pos.col, inserted_chars);
            lines[pos.row] = chars.into_iter().collect();
            return Ok(CursorPosition::new(pos.row, pos.col + added_count));
        }

        // Multi-line insert
        let current_line = &lines[pos.row];
        let current_chars: Vec<char> = current_line.chars().collect();
        let prefix: String = current_chars[..pos.col.min(current_chars.len())]
            .iter()
            .collect();
        let suffix: String = current_chars[pos.col.min(current_chars.len())..]
            .iter()
            .collect();

        let first_new_line = format!("{}{}", prefix, input_lines[0]);
        lines[pos.row] = first_new_line;

        let mut current_row = pos.row + 1;
        for i in 1..input_lines.len() - 1 {
            lines.insert(current_row, input_lines[i].to_string());
            current_row += 1;
        }

        let last_input = input_lines.last().unwrap();
        let last_new_line = format!("{}{}", last_input, suffix);
        lines.insert(current_row, last_new_line);

        let final_col = last_input.chars().count();
        Ok(CursorPosition::new(current_row, final_col))
    }

    /// Deletes character at specified cursor position.
    /// Returns the deleted character.
    pub fn delete_char(lines: &mut Vec<String>, pos: CursorPosition) -> Result<char, BufferError> {
        if lines.is_empty() || pos.row >= lines.len() {
            return Err(BufferError::InvalidPosition {
                row: pos.row,
                col: pos.col,
            });
        }

        let line = &mut lines[pos.row];
        let mut chars: Vec<char> = line.chars().collect();

        if pos.col >= chars.len() {
            return Err(BufferError::InvalidPosition {
                row: pos.row,
                col: pos.col,
            });
        }

        let deleted = chars.remove(pos.col);
        lines[pos.row] = chars.into_iter().collect();
        Ok(deleted)
    }

    /// Deletes text range `[start, end)` from lines vector.
    /// Returns deleted string.
    pub fn delete_range(
        lines: &mut Vec<String>,
        start: CursorPosition,
        end: CursorPosition,
    ) -> Result<String, BufferError> {
        if lines.is_empty() {
            return Err(BufferError::EmptyBuffer);
        }

        let (s, e) = if start <= end {
            (start, end)
        } else {
            (end, start)
        };

        if s.row >= lines.len() {
            return Err(BufferError::InvalidPosition {
                row: s.row,
                col: s.col,
            });
        }

        let e_row = e.row.min(lines.len() - 1);

        if s.row == e_row {
            let line = &mut lines[s.row];
            let mut chars: Vec<char> = line.chars().collect();
            let s_col = s.col.min(chars.len());
            let e_col = e.col.min(chars.len());

            let deleted_chars: Vec<char> = chars.drain(s_col..e_col).collect();
            lines[s.row] = chars.into_iter().collect();
            return Ok(deleted_chars.into_iter().collect());
        }

        // Multi-line range deletion
        let mut deleted_parts = Vec::new();

        let first_line = &lines[s.row];
        let first_chars: Vec<char> = first_line.chars().collect();
        let s_col = s.col.min(first_chars.len());
        let first_prefix: String = first_chars[..s_col].iter().collect();
        let first_deleted: String = first_chars[s_col..].iter().collect();
        deleted_parts.push(first_deleted);

        for r in (s.row + 1)..e_row {
            deleted_parts.push(lines[r].clone());
        }

        let last_line = &lines[e_row];
        let last_chars: Vec<char> = last_line.chars().collect();
        let e_col = e.col.min(last_chars.len());
        let last_deleted: String = last_chars[..e_col].iter().collect();
        let last_suffix: String = last_chars[e_col..].iter().collect();
        deleted_parts.push(last_deleted);

        // Remove intermediate lines
        lines.drain((s.row + 1)..=e_row);

        // Combine prefix of first line with suffix of last line
        lines[s.row] = format!("{}{}", first_prefix, last_suffix);

        Ok(deleted_parts.join("\n"))
    }

    /// Splits line at specified position into two lines.
    pub fn split_line(lines: &mut Vec<String>, pos: CursorPosition) -> Result<(), BufferError> {
        if lines.is_empty() {
            lines.push(String::new());
            lines.push(String::new());
            return Ok(());
        }

        if pos.row >= lines.len() {
            return Err(BufferError::InvalidPosition {
                row: pos.row,
                col: pos.col,
            });
        }

        let current_line = &lines[pos.row];
        let chars: Vec<char> = current_line.chars().collect();
        let split_col = pos.col.min(chars.len());

        let line1: String = chars[..split_col].iter().collect();
        let line2: String = chars[split_col..].iter().collect();

        lines[pos.row] = line1;
        lines.insert(pos.row + 1, line2);

        Ok(())
    }

    /// Merges line at `row` with line `row + 1`.
    /// Returns column offset where second line was appended.
    pub fn merge_lines(lines: &mut Vec<String>, row: usize) -> Result<usize, BufferError> {
        if lines.is_empty() || row + 1 >= lines.len() {
            return Err(BufferError::InvalidPosition { row, col: 0 });
        }

        let second_line = lines.remove(row + 1);
        let first_line = &mut lines[row];
        let col_offset = first_line.chars().count();

        first_line.push_str(&second_line);

        Ok(col_offset)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_char_and_str() {
        let mut lines = vec!["Hello".into()];
        BufferOperations::insert_char(&mut lines, CursorPosition::new(0, 5), '!').unwrap();
        assert_eq!(lines[0], "Hello!");

        BufferOperations::insert_str(&mut lines, CursorPosition::new(0, 6), " World\nSecond")
            .unwrap();
        assert_eq!(lines[0], "Hello! World");
        assert_eq!(lines[1], "Second");
    }

    #[test]
    fn test_split_and_merge_lines() {
        let mut lines = vec!["Hello World".into()];

        BufferOperations::split_line(&mut lines, CursorPosition::new(0, 5)).unwrap();
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0], "Hello");
        assert_eq!(lines[1], " World");

        let offset = BufferOperations::merge_lines(&mut lines, 0).unwrap();
        assert_eq!(offset, 5);
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0], "Hello World");
    }

    #[test]
    fn test_delete_range() {
        let mut lines = vec!["Line 1".into(), "Line 2".into(), "Line 3".into()];
        let deleted = BufferOperations::delete_range(
            &mut lines,
            CursorPosition::new(0, 4),
            CursorPosition::new(2, 4),
        )
        .unwrap();

        assert_eq!(deleted, " 1\nLine 2\nLine");
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0], "Line 3");
    }
}
