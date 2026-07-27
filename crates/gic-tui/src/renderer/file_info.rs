//! # File Information
//!
//! Extracts display-ready file information from `Document` and `TextBuffer`
//! for use in the status bar and other UI elements. This is a pure data
//! extraction module — no rendering.

use gic_core::{Document, TextBuffer};

/// Display-ready file information extracted from application state.
///
/// This struct is a read-only snapshot of file metadata formatted for
/// UI display. It is constructed from immutable references and can be
/// passed to any rendering component.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileInfo {
    /// File name for display (e.g., "main.rs"), or "\[Untitled\]".
    pub file_name: String,
    /// File extension (e.g., "rs"), or empty string.
    pub extension: String,
    /// Character encoding (e.g., "UTF-8").
    pub encoding: String,
    /// Line ending type (e.g., "LF", "CRLF").
    pub line_ending: String,
    /// Whether the file is read-only.
    pub is_read_only: bool,
    /// File size in bytes.
    pub size_bytes: u64,
    /// Whether the buffer has unsaved modifications.
    pub is_modified: bool,
    /// Total number of lines.
    pub line_count: usize,
    /// Current cursor row (0-indexed).
    pub cursor_row: usize,
    /// Current cursor column (0-indexed).
    pub cursor_col: usize,
    /// Detected or configured language name.
    pub language: String,
}

impl FileInfo {
    /// Extracts file information from a document, text buffer, and cursor position.
    ///
    /// # Arguments
    ///
    /// * `document` - The open document with file metadata.
    /// * `buffer` - The text buffer containing file content.
    /// * `cursor_row` - Current cursor row (0-indexed).
    /// * `cursor_col` - Current cursor column (0-indexed).
    /// * `language` - Optional detected language name.
    pub fn from_state(
        document: &Document,
        buffer: &TextBuffer,
        cursor_row: usize,
        cursor_col: usize,
        language: Option<&str>,
    ) -> Self {
        let file_name = document
            .path
            .as_ref()
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "[Untitled]".to_string());

        let extension = document
            .path
            .as_ref()
            .and_then(|p| p.extension())
            .map(|e| e.to_string_lossy().to_string())
            .unwrap_or_default();

        Self {
            file_name,
            extension,
            encoding: document.metadata.encoding.clone(),
            line_ending: "LF".to_string(), // Default; future milestone will detect
            is_read_only: document.is_read_only,
            size_bytes: document.metadata.size_bytes,
            is_modified: buffer.is_modified() || document.is_modified,
            line_count: buffer.line_count(),
            cursor_row,
            cursor_col,
            language: language.unwrap_or("Plain Text").to_string(),
        }
    }

    /// Returns the cursor position as a display string (1-indexed).
    ///
    /// Format: "Ln N, Col M"
    pub fn cursor_display(&self) -> String {
        format!("Ln {}, Col {}", self.cursor_row + 1, self.cursor_col + 1)
    }

    /// Returns a human-readable file size string.
    pub fn size_display(&self) -> String {
        if self.size_bytes < 1024 {
            format!("{} B", self.size_bytes)
        } else if self.size_bytes < 1024 * 1024 {
            format!("{:.1} KB", self.size_bytes as f64 / 1024.0)
        } else {
            format!("{:.1} MB", self.size_bytes as f64 / (1024.0 * 1024.0))
        }
    }

    /// Returns the total line count as a display string.
    pub fn lines_display(&self) -> String {
        format!("{} lines", self.line_count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gic_core::{Document, TextBuffer};

    #[test]
    fn test_file_info_from_empty_document() {
        let doc = Document::new_empty();
        let buffer = TextBuffer::new();

        let info = FileInfo::from_state(&doc, &buffer, 0, 0, None);

        assert_eq!(info.file_name, "[Untitled]");
        assert_eq!(info.extension, "");
        assert_eq!(info.encoding, "UTF-8");
        assert_eq!(info.line_ending, "LF");
        assert!(!info.is_read_only);
        assert!(!info.is_modified);
        assert_eq!(info.line_count, 1);
        assert_eq!(info.language, "Plain Text");
    }

    #[test]
    fn test_file_info_from_document_with_path() {
        let mut doc = Document::new_empty();
        doc.set_path("z:/projects/main.rs");

        let buffer = TextBuffer::from_str("fn main() {\n    println!(\"Hello\");\n}");

        let info = FileInfo::from_state(&doc, &buffer, 1, 4, Some("Rust"));

        assert_eq!(info.file_name, "main.rs");
        assert_eq!(info.extension, "rs");
        assert_eq!(info.language, "Rust");
        assert_eq!(info.line_count, 3);
        assert_eq!(info.cursor_row, 1);
        assert_eq!(info.cursor_col, 4);
    }

    #[test]
    fn test_cursor_display() {
        let doc = Document::new_empty();
        let buffer = TextBuffer::new();
        let info = FileInfo::from_state(&doc, &buffer, 9, 14, None);

        assert_eq!(info.cursor_display(), "Ln 10, Col 15"); // 1-indexed
    }

    #[test]
    fn test_size_display() {
        let doc = Document::new_empty();
        let buffer = TextBuffer::new();

        let mut info = FileInfo::from_state(&doc, &buffer, 0, 0, None);
        info.size_bytes = 500;
        assert_eq!(info.size_display(), "500 B");

        info.size_bytes = 2048;
        assert_eq!(info.size_display(), "2.0 KB");

        info.size_bytes = 1_500_000;
        assert_eq!(info.size_display(), "1.4 MB");
    }

    #[test]
    fn test_modified_detection() {
        let doc = Document::new_empty();
        let mut buffer = TextBuffer::new();
        buffer.insert_str("hello").unwrap();

        let info = FileInfo::from_state(&doc, &buffer, 0, 5, None);
        assert!(info.is_modified);
    }

    #[test]
    fn test_read_only() {
        let mut doc = Document::new_empty();
        doc.is_read_only = true;
        let buffer = TextBuffer::new();

        let info = FileInfo::from_state(&doc, &buffer, 0, 0, None);
        assert!(info.is_read_only);
    }
}
