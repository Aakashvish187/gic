use crate::buffer::cursor::CursorPosition;
use serde::{Deserialize, Serialize};

/// Atomic mutation commands for text buffer editing and command-pattern undo/redo.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BufferCommand {
    /// Inserts a single character at specified cursor position.
    InsertChar { pos: CursorPosition, ch: char },
    /// Inserts string text (single line or multiline) at specified cursor position.
    InsertText { pos: CursorPosition, text: String },
    /// Deletes a single character at specified cursor position.
    DeleteChar {
        pos: CursorPosition,
        deleted_ch: char,
    },
    /// Deletes text within range `[start, end)`.
    DeleteText {
        start: CursorPosition,
        end: CursorPosition,
        deleted_text: String,
    },
    /// Splits line at specified cursor position into two lines.
    SplitLine { pos: CursorPosition },
    /// Merges line at `row` with line `row + 1` at column offset `col_offset`.
    MergeLines { row: usize, col_offset: usize },
}

impl BufferCommand {
    /// Generates the inverse command required to undo this mutation.
    pub fn inverse(&self) -> Self {
        match self {
            BufferCommand::InsertChar { pos, ch } => BufferCommand::DeleteChar {
                pos: *pos,
                deleted_ch: *ch,
            },
            BufferCommand::InsertText { pos, text } => {
                let lines: Vec<&str> = text.lines().collect();
                let end_pos = if lines.len() <= 1 {
                    let char_count = text.chars().count();
                    CursorPosition::new(pos.row, pos.col + char_count)
                } else {
                    let last_line_len = lines.last().map(|l| l.chars().count()).unwrap_or(0);
                    CursorPosition::new(pos.row + lines.len() - 1, last_line_len)
                };
                BufferCommand::DeleteText {
                    start: *pos,
                    end: end_pos,
                    deleted_text: text.clone(),
                }
            }
            BufferCommand::DeleteChar { pos, deleted_ch } => BufferCommand::InsertChar {
                pos: *pos,
                ch: *deleted_ch,
            },
            BufferCommand::DeleteText {
                start,
                deleted_text,
                ..
            } => BufferCommand::InsertText {
                pos: *start,
                text: deleted_text.clone(),
            },
            BufferCommand::SplitLine { pos } => BufferCommand::MergeLines {
                row: pos.row,
                col_offset: pos.col,
            },
            BufferCommand::MergeLines { row, col_offset } => BufferCommand::SplitLine {
                pos: CursorPosition::new(*row, *col_offset),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer_command_inverse() {
        let pos = CursorPosition::new(1, 4);

        let ins_char = BufferCommand::InsertChar { pos, ch: 'A' };
        let inv_char = ins_char.inverse();
        assert_eq!(
            inv_char,
            BufferCommand::DeleteChar {
                pos,
                deleted_ch: 'A'
            }
        );

        let split = BufferCommand::SplitLine { pos };
        let inv_split = split.inverse();
        assert_eq!(
            inv_split,
            BufferCommand::MergeLines {
                row: 1,
                col_offset: 4
            }
        );
    }
}
