//! Text Buffer Engine for GIC (General Infrastructure Console).
//!
//! Independent core module responsible for storing, editing, selecting,
//! clipboard operations, and command-pattern undo/redo history management.

pub mod clipboard;
pub mod commands;
pub mod cursor;
pub mod errors;
pub mod history;
pub mod operations;
pub mod selection;
pub mod text_buffer;

pub use clipboard::{ClipboardContentType, InternalClipboard};
pub use commands::BufferCommand;
pub use cursor::{Cursor, CursorPosition};
pub use errors::BufferError;
pub use history::{CommandGroup, UndoRedoHistory};
pub use operations::BufferOperations;
pub use selection::{Selection, SelectionMode};
pub use text_buffer::TextBuffer;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unicode_and_emoji_handling() {
        let mut buffer = TextBuffer::new();
        // Insert multibyte UTF-8 and Emojis: 🦀 (Rust), ⚡ (Lightning), 🚀 (Rocket)
        buffer.insert_str("Rust 🦀 ⚡ 🚀").unwrap();
        assert_eq!(buffer.text(), "Rust 🦀 ⚡ 🚀");

        // Character count in line: "Rust 🦀 ⚡ 🚀" -> 11 chars
        assert_eq!(buffer.line(0).unwrap().chars().count(), 10);

        // Move cursor back 3 characters (land before ⚡)
        buffer.move_left();
        buffer.move_left();
        buffer.move_left();
        assert_eq!(buffer.cursor().col(), 7);

        // Delete char at cursor position
        buffer.delete_char().unwrap();
        assert_eq!(buffer.text(), "Rust 🦀  🚀");
    }

    #[test]
    fn test_large_buffer_performance() {
        let line_count = 10_000;
        let mut lines = Vec::with_capacity(line_count);
        for i in 0..line_count {
            lines.push(format!("server_config_row_{}_setting=true", i));
        }

        let mut buffer = TextBuffer::from_lines(lines);
        assert_eq!(buffer.line_count(), line_count);

        // Navigate to middle
        buffer.set_cursor_position(5000, 10);
        assert_eq!(buffer.cursor().row(), 5000);
        assert_eq!(buffer.cursor().col(), 10);

        // Edit at middle
        buffer.insert_str("_edited").unwrap();
        assert!(buffer.line(5000).unwrap().contains("_edited"));

        // Undo edit
        buffer.undo().unwrap();
        assert!(!buffer.line(5000).unwrap().contains("_edited"));
    }

    #[test]
    fn test_boundary_empty_and_blank_lines() {
        let mut buffer = TextBuffer::new();
        assert!(buffer.is_empty());

        buffer.move_left();
        buffer.move_right();
        buffer.move_up();
        buffer.move_down();
        assert_eq!(buffer.cursor().position, CursorPosition::zero());

        buffer.insert_newline().unwrap();
        assert_eq!(buffer.line_count(), 2);
        assert_eq!(buffer.text(), "\n");

        buffer.undo().unwrap();
        assert_eq!(buffer.line_count(), 1);
        assert_eq!(buffer.text(), "");
    }
}
