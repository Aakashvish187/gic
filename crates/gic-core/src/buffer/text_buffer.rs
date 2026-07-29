use crate::buffer::clipboard::{ClipboardContentType, InternalClipboard};
use crate::buffer::commands::BufferCommand;
use crate::buffer::cursor::{Cursor, CursorPosition};
use crate::buffer::errors::BufferError;
use crate::buffer::history::{CommandGroup, UndoRedoHistory};
use crate::buffer::operations::BufferOperations;
use crate::buffer::selection::{Selection, SelectionMode};
use serde::{Deserialize, Serialize};

/// High-performance production-ready Text Buffer Engine for GIC editor.
/// Stores text as a vector of line strings (`Vec<String>`), supporting
/// cursor navigation, selections, clipboard, and transaction undo/redo history.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextBuffer {
    lines: Vec<String>,
    cursor: Cursor,
    selection: Selection,
    clipboard: InternalClipboard,
    history: UndoRedoHistory,
    is_modified: bool,
}

impl Default for TextBuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl TextBuffer {
    /// Creates a new empty `TextBuffer` initialized with one empty line.
    pub fn new() -> Self {
        Self {
            lines: vec![String::new()],
            cursor: Cursor::new(),
            selection: Selection::new(),
            clipboard: InternalClipboard::new(),
            history: UndoRedoHistory::default(),
            is_modified: false,
        }
    }

    /// Creates a `TextBuffer` from an initial text string.
    pub fn from_str(text: &str) -> Self {
        let lines: Vec<String> = if text.is_empty() {
            vec![String::new()]
        } else {
            text.split('\n')
                .map(|s| s.trim_end_matches('\r').to_string())
                .collect()
        };

        Self {
            lines,
            cursor: Cursor::new(),
            selection: Selection::new(),
            clipboard: InternalClipboard::new(),
            history: UndoRedoHistory::default(),
            is_modified: false,
        }
    }

    /// Creates a `TextBuffer` from a vector of line strings.
    pub fn from_lines(lines: Vec<String>) -> Self {
        let safe_lines = if lines.is_empty() {
            vec![String::new()]
        } else {
            lines
        };

        Self {
            lines: safe_lines,
            cursor: Cursor::new(),
            selection: Selection::new(),
            clipboard: InternalClipboard::new(),
            history: UndoRedoHistory::default(),
            is_modified: false,
        }
    }

    // --- Line & Text Accessors ---

    /// Returns reference to all lines.
    pub fn lines(&self) -> &[String] {
        &self.lines
    }

    /// Returns total line count.
    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    /// Returns reference to line string at `row` index if present.
    pub fn line(&self, row: usize) -> Option<&str> {
        self.lines.get(row).map(|s| s.as_str())
    }

    pub fn text(&self) -> String {
        self.lines.join("\n")
    }

    /// Sets the entire buffer text to the new content.
    pub fn set_text(&mut self, text: &str) {
        let lines: Vec<String> = if text.is_empty() {
            vec![String::new()]
        } else {
            text.split('\n')
                .map(|s| s.trim_end_matches('\r').to_string())
                .collect()
        };
        self.lines = lines;
        self.cursor.clamp(&self.lines);
        self.is_modified = true;
    }

    /// Returns true if buffer contains no text.
    pub fn is_empty(&self) -> bool {
        self.lines.is_empty() || (self.lines.len() == 1 && self.lines[0].is_empty())
    }

    /// Returns true if buffer has unpersisted modifications.
    pub fn is_modified(&self) -> bool {
        self.is_modified
    }

    /// Clears buffer content.
    pub fn clear(&mut self) {
        self.lines = vec![String::new()];
        self.cursor = Cursor::new();
        self.selection.clear();
        self.history.clear();
        self.is_modified = false;
    }

    // --- Cursor Navigation ---

    /// Access reference to cursor.
    pub fn cursor(&self) -> &Cursor {
        &self.cursor
    }

    /// Access mutable reference to cursor.
    pub fn cursor_mut(&mut self) -> &mut Cursor {
        &mut self.cursor
    }

    /// Returns active 0-indexed cursor position (row, col).
    pub fn cursor_position(&self) -> CursorPosition {
        self.cursor.position
    }

    /// Sets explicit cursor position, clamping to valid bounds.
    pub fn set_cursor_position(&mut self, row: usize, col: usize) {
        self.cursor.position = CursorPosition::new(row, col);
        self.cursor.clamp(&self.lines);
        if self.selection.is_active {
            self.selection.update(self.cursor.position);
        }
    }

    pub fn move_left(&mut self) {
        self.cursor.move_left(&self.lines);
        if self.selection.is_active {
            self.selection.update(self.cursor.position);
        }
    }

    pub fn move_right(&mut self) {
        self.cursor.move_right(&self.lines);
        if self.selection.is_active {
            self.selection.update(self.cursor.position);
        }
    }

    pub fn move_up(&mut self) {
        self.cursor.move_up(&self.lines);
        if self.selection.is_active {
            self.selection.update(self.cursor.position);
        }
    }

    pub fn move_down(&mut self) {
        self.cursor.move_down(&self.lines);
        if self.selection.is_active {
            self.selection.update(self.cursor.position);
        }
    }

    pub fn move_to_line_start(&mut self) {
        self.cursor.move_to_line_start();
        if self.selection.is_active {
            self.selection.update(self.cursor.position);
        }
    }

    pub fn move_to_line_end(&mut self) {
        self.cursor.move_to_line_end(&self.lines);
        if self.selection.is_active {
            self.selection.update(self.cursor.position);
        }
    }

    pub fn move_to_file_start(&mut self) {
        self.cursor.move_to_file_start();
        if self.selection.is_active {
            self.selection.update(self.cursor.position);
        }
    }

    pub fn move_to_file_end(&mut self) {
        self.cursor.move_to_file_end(&self.lines);
        if self.selection.is_active {
            self.selection.update(self.cursor.position);
        }
    }

    pub fn move_word_forward(&mut self) {
        self.cursor.move_word_forward(&self.lines);
        if self.selection.is_active {
            self.selection.update(self.cursor.position);
        }
    }

    pub fn move_word_backward(&mut self) {
        self.cursor.move_word_backward(&self.lines);
        if self.selection.is_active {
            self.selection.update(self.cursor.position);
        }
    }

    // --- Insert Operations ---

    /// Inserts a single character at current cursor position.
    pub fn insert_char(&mut self, ch: char) -> Result<(), BufferError> {
        if self.selection.is_active {
            let _ = self.delete_selection();
        }

        let pos_before = self.cursor.position;
        let mut group = CommandGroup::new(
            pos_before,
            if self.selection.is_active {
                Some(self.selection)
            } else {
                None
            },
        );

        if ch == '\n' {
            BufferOperations::split_line(&mut self.lines, pos_before)?;
            group.add_command(BufferCommand::SplitLine { pos: pos_before });
            self.cursor.position = CursorPosition::new(pos_before.row + 1, 0);
        } else {
            BufferOperations::insert_char(&mut self.lines, pos_before, ch)?;
            group.add_command(BufferCommand::InsertChar {
                pos: pos_before,
                ch,
            });
            self.cursor.position.col += 1;
        }

        self.is_modified = true;
        group.finalize(self.cursor.position, None);
        self.history.push_group(group);
        Ok(())
    }

    /// Inserts text string at current cursor position.
    pub fn insert_str(&mut self, text: &str) -> Result<(), BufferError> {
        if text.is_empty() {
            return Ok(());
        }

        if self.selection.is_active {
            let _ = self.delete_selection();
        }

        let pos_before = self.cursor.position;
        let mut group = CommandGroup::new(pos_before, None);

        let new_pos = BufferOperations::insert_str(&mut self.lines, pos_before, text)?;
        group.add_command(BufferCommand::InsertText {
            pos: pos_before,
            text: text.to_string(),
        });

        self.cursor.position = new_pos;
        self.is_modified = true;
        group.finalize(self.cursor.position, None);
        self.history.push_group(group);
        Ok(())
    }

    /// Inserts a newline at current cursor position.
    pub fn insert_newline(&mut self) -> Result<(), BufferError> {
        self.insert_char('\n')
    }

    /// Inserts tabs or equivalent spaces at current cursor position.
    pub fn insert_tab(&mut self, tab_width: usize) -> Result<(), BufferError> {
        let spaces = " ".repeat(if tab_width == 0 { 4 } else { tab_width });
        self.insert_str(&spaces)
    }

    // --- Delete Operations ---

    /// Deletes character at cursor position.
    pub fn delete_char(&mut self) -> Result<(), BufferError> {
        if self.selection.is_active {
            return self.delete_selection();
        }

        let pos = self.cursor.position;
        let current_line_len = self
            .lines
            .get(pos.row)
            .map(|l| l.chars().count())
            .unwrap_or(0);

        let mut group = CommandGroup::new(pos, None);

        if pos.col < current_line_len {
            let deleted_ch = BufferOperations::delete_char(&mut self.lines, pos)?;
            group.add_command(BufferCommand::DeleteChar { pos, deleted_ch });
        } else if pos.row + 1 < self.lines.len() {
            let col_offset = BufferOperations::merge_lines(&mut self.lines, pos.row)?;
            group.add_command(BufferCommand::MergeLines {
                row: pos.row,
                col_offset,
            });
        } else {
            return Ok(());
        }

        self.is_modified = true;
        group.finalize(self.cursor.position, None);
        self.history.push_group(group);
        Ok(())
    }

    /// Deletes character before cursor (Backspace). Merges line with previous if at col 0.
    pub fn delete_backspace(&mut self) -> Result<(), BufferError> {
        if self.selection.is_active {
            return self.delete_selection();
        }

        if self.cursor.position == CursorPosition::zero() {
            return Ok(());
        }

        if self.cursor.position.col > 0 {
            self.cursor.position.col -= 1;
            self.delete_char()
        } else if self.cursor.position.row > 0 {
            let prev_row = self.cursor.position.row - 1;
            let prev_len = self
                .lines
                .get(prev_row)
                .map(|l| l.chars().count())
                .unwrap_or(0);
            self.cursor.position = CursorPosition::new(prev_row, prev_len);

            let mut group = CommandGroup::new(CursorPosition::new(prev_row + 1, 0), None);
            let col_offset = BufferOperations::merge_lines(&mut self.lines, prev_row)?;
            group.add_command(BufferCommand::MergeLines {
                row: prev_row,
                col_offset,
            });

            self.is_modified = true;
            group.finalize(self.cursor.position, None);
            self.history.push_group(group);
            Ok(())
        } else {
            Ok(())
        }
    }

    /// Deletes entire line at current cursor row.
    pub fn delete_line(&mut self) -> Result<(), BufferError> {
        let row = self.cursor.position.row;
        if self.lines.is_empty() || row >= self.lines.len() {
            return Ok(());
        }

        let start = CursorPosition::new(row, 0);
        let line_len = self.lines[row].chars().count();
        let end = if self.lines.len() > 1 && row + 1 < self.lines.len() {
            CursorPosition::new(row + 1, 0)
        } else {
            CursorPosition::new(row, line_len)
        };

        self.delete_range(start, end)
    }

    /// Deletes word forward or backward.
    pub fn delete_word(&mut self) -> Result<(), BufferError> {
        let start_pos = self.cursor.position;
        self.move_word_forward();
        let end_pos = self.cursor.position;
        self.cursor.position = start_pos;
        self.delete_range(start_pos, end_pos)
    }

    /// Deletes active selected text range.
    pub fn delete_selection(&mut self) -> Result<(), BufferError> {
        if !self.selection.is_active {
            return Err(BufferError::InvalidSelection);
        }

        let (start, end) = self.selection.range();
        self.selection.clear();
        self.delete_range(start, end)
    }

    /// Deletes range `[start, end)`.
    pub fn delete_range(
        &mut self,
        start: CursorPosition,
        end: CursorPosition,
    ) -> Result<(), BufferError> {
        if start == end {
            return Ok(());
        }

        let (s, e) = if start <= end {
            (start, end)
        } else {
            (end, start)
        };
        let pos_before = self.cursor.position;
        let mut group = CommandGroup::new(pos_before, None);

        let deleted_text = BufferOperations::delete_range(&mut self.lines, s, e)?;
        group.add_command(BufferCommand::DeleteText {
            start: s,
            end: e,
            deleted_text,
        });

        self.cursor.position = s;
        self.cursor.clamp(&self.lines);
        self.is_modified = true;
        group.finalize(self.cursor.position, None);
        self.history.push_group(group);
        Ok(())
    }

    /// Replaces text in range `[start, end)` with `replacement` as a single atomic transaction.
    pub fn replace_range(
        &mut self,
        start: CursorPosition,
        end: CursorPosition,
        replacement: &str,
    ) -> Result<(), BufferError> {
        let (s, e) = if start <= end {
            (start, end)
        } else {
            (end, start)
        };
        let pos_before = self.cursor.position;
        let mut group = CommandGroup::new(pos_before, None);

        if s != e {
            let deleted_text = BufferOperations::delete_range(&mut self.lines, s, e)?;
            group.add_command(BufferCommand::DeleteText {
                start: s,
                end: e,
                deleted_text,
            });
        }

        if !replacement.is_empty() {
            let new_pos = BufferOperations::insert_str(&mut self.lines, s, replacement)?;
            group.add_command(BufferCommand::InsertText {
                pos: s,
                text: replacement.to_string(),
            });
            self.cursor.position = new_pos;
        } else {
            self.cursor.position = s;
        }

        self.cursor.clamp(&self.lines);
        self.is_modified = true;
        group.finalize(self.cursor.position, None);
        self.history.push_group(group);
        Ok(())
    }

    // --- Selection System ---

    /// Access reference to selection state.
    pub fn selection(&self) -> &Selection {
        &self.selection
    }

    /// Access mutable reference to selection state.
    pub fn selection_mut(&mut self) -> &mut Selection {
        &mut self.selection
    }

    /// Starts selection at current cursor position.
    pub fn start_selection(&mut self, mode: SelectionMode) {
        self.selection.start(self.cursor.position, mode);
    }

    /// Updates dynamic head of selection to current cursor position.
    pub fn update_selection(&mut self) {
        self.selection.update(self.cursor.position);
    }

    /// Clears selection.
    pub fn clear_selection(&mut self) {
        self.selection.clear();
    }

    /// Returns selected text if active selection exists.
    pub fn selected_text(&self) -> Option<String> {
        self.selection.get_selected_text(&self.lines)
    }

    // --- Clipboard Operations ---

    /// Access reference to internal clipboard.
    pub fn clipboard(&self) -> &InternalClipboard {
        &self.clipboard
    }

    /// Copies selected text (or current line if no selection) to internal clipboard.
    pub fn copy(&mut self) -> Result<(), BufferError> {
        if self.selection.is_active {
            if let Some(text) = self.selected_text() {
                self.clipboard.set_text(text);
                return Ok(());
            }
        }

        let current_row = self.cursor.position.row;
        if let Some(line) = self.lines.get(current_row) {
            self.clipboard.set_lines(vec![line.clone()]);
        }
        Ok(())
    }

    /// Cuts selected text to internal clipboard and deletes range.
    pub fn cut(&mut self) -> Result<(), BufferError> {
        self.copy()?;
        if self.selection.is_active {
            self.delete_selection()?;
        } else {
            self.delete_line()?;
        }
        Ok(())
    }

    /// Pastes clipboard content at current cursor position.
    pub fn paste(&mut self) -> Result<(), BufferError> {
        let content = self
            .clipboard
            .get_content()
            .cloned()
            .ok_or(BufferError::ClipboardEmpty)?;

        match content {
            ClipboardContentType::Character(text) => self.insert_str(&text),
            ClipboardContentType::Line(lines) => self.insert_str(&lines.join("\n")),
        }
    }

    /// Pastes clipboard content as a new line above current row.
    pub fn paste_above(&mut self) -> Result<(), BufferError> {
        let content = self
            .clipboard
            .get_content()
            .cloned()
            .ok_or(BufferError::ClipboardEmpty)?;

        let target_row = self.cursor.position.row;
        self.lines.insert(target_row, content.as_str_content());
        self.cursor.position = CursorPosition::new(target_row, 0);
        self.is_modified = true;
        Ok(())
    }

    /// Pastes clipboard content as a new line below current row.
    pub fn paste_below(&mut self) -> Result<(), BufferError> {
        let content = self
            .clipboard
            .get_content()
            .cloned()
            .ok_or(BufferError::ClipboardEmpty)?;

        let target_row = (self.cursor.position.row + 1).min(self.lines.len());
        self.lines.insert(target_row, content.as_str_content());
        self.cursor.position = CursorPosition::new(target_row, 0);
        self.is_modified = true;
        Ok(())
    }

    // --- Undo / Redo System ---

    /// Returns true if undo operations are available.
    pub fn can_undo(&self) -> bool {
        self.history.can_undo()
    }

    /// Returns true if redo operations are available.
    pub fn can_redo(&self) -> bool {
        self.history.can_redo()
    }

    /// Reverses the last command transaction group.
    pub fn undo(&mut self) -> Result<(), BufferError> {
        let group = self.history.pop_undo().ok_or(BufferError::HistoryEmpty)?;

        // Execute inverse commands in reverse order
        for cmd in group.commands.iter().rev() {
            let inv = cmd.inverse();
            self.apply_command(&inv)?;
        }

        self.cursor.position = group.cursor_before;
        self.cursor.clamp(&self.lines);
        if let Some(sel) = group.selection_before {
            self.selection = sel;
        } else {
            self.selection.clear();
        }
        self.is_modified = true;
        Ok(())
    }

    /// Re-applies next command transaction group.
    pub fn redo(&mut self) -> Result<(), BufferError> {
        let group = self.history.pop_redo().ok_or(BufferError::HistoryEmpty)?;

        for cmd in &group.commands {
            self.apply_command(cmd)?;
        }

        self.cursor.position = group.cursor_after;
        self.cursor.clamp(&self.lines);
        if let Some(sel) = group.selection_after {
            self.selection = sel;
        } else {
            self.selection.clear();
        }
        self.is_modified = true;
        Ok(())
    }

    /// Applies a command directly to storage without pushing to history stack.
    fn apply_command(&mut self, cmd: &BufferCommand) -> Result<(), BufferError> {
        match cmd {
            BufferCommand::InsertChar { pos, ch } => {
                BufferOperations::insert_char(&mut self.lines, *pos, *ch)?;
            }
            BufferCommand::InsertText { pos, text } => {
                BufferOperations::insert_str(&mut self.lines, *pos, text)?;
            }
            BufferCommand::DeleteChar { pos, .. } => {
                BufferOperations::delete_char(&mut self.lines, *pos)?;
            }
            BufferCommand::DeleteText { start, end, .. } => {
                BufferOperations::delete_range(&mut self.lines, *start, *end)?;
            }
            BufferCommand::SplitLine { pos } => {
                BufferOperations::split_line(&mut self.lines, *pos)?;
            }
            BufferCommand::MergeLines { row, .. } => {
                BufferOperations::merge_lines(&mut self.lines, *row)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_buffer_insert_delete_workflow() {
        let mut buffer = TextBuffer::new();
        buffer.insert_str("Hello World").unwrap();
        assert_eq!(buffer.text(), "Hello World");
        assert_eq!(buffer.cursor().col(), 11);

        buffer.delete_backspace().unwrap();
        assert_eq!(buffer.text(), "Hello Worl");

        buffer.insert_newline().unwrap();
        assert_eq!(buffer.line_count(), 2);
        assert_eq!(buffer.line(0), Some("Hello Worl"));
        assert_eq!(buffer.line(1), Some(""));
    }

    #[test]
    fn test_text_buffer_undo_redo() {
        let mut buffer = TextBuffer::new();
        buffer.insert_str("Initial").unwrap();
        assert_eq!(buffer.text(), "Initial");

        buffer.insert_str(" Second").unwrap();
        assert_eq!(buffer.text(), "Initial Second");

        buffer.undo().unwrap();
        assert_eq!(buffer.text(), "Initial");

        buffer.redo().unwrap();
        assert_eq!(buffer.text(), "Initial Second");
    }

    #[test]
    fn test_text_buffer_selection_and_copy_paste() {
        let mut buffer = TextBuffer::from_str("Copy Target Text");
        buffer.set_cursor_position(0, 5);
        buffer.start_selection(SelectionMode::Character);
        buffer.set_cursor_position(0, 11);

        buffer.copy().unwrap();
        buffer.clear_selection();

        buffer.move_to_file_end();
        buffer.paste().unwrap();
        assert_eq!(buffer.text(), "Copy Target TextTarget");
    }
}
