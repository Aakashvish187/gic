use serde::{Deserialize, Serialize};

/// Type of content stored in the internal clipboard.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClipboardContentType {
    /// In-line character selection text.
    Character(String),
    /// Whole line or multi-line list.
    Line(Vec<String>),
}

impl ClipboardContentType {
    /// Returns text string representation of clipboard content.
    pub fn as_str_content(&self) -> String {
        match self {
            ClipboardContentType::Character(s) => s.clone(),
            ClipboardContentType::Line(lines) => lines.join("\n"),
        }
    }
}

/// Internal in-memory clipboard for text buffer operations (copy, cut, paste, paste above/below).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct InternalClipboard {
    content: Option<ClipboardContentType>,
}

impl InternalClipboard {
    /// Creates a new empty `InternalClipboard`.
    pub fn new() -> Self {
        Self { content: None }
    }

    /// Stores character/inline text in clipboard.
    pub fn set_text<S: Into<String>>(&mut self, text: S) {
        let t = text.into();
        if t.is_empty() {
            self.content = None;
        } else {
            self.content = Some(ClipboardContentType::Character(t));
        }
    }

    /// Stores full lines in clipboard.
    pub fn set_lines(&mut self, lines: Vec<String>) {
        if lines.is_empty() {
            self.content = None;
        } else {
            self.content = Some(ClipboardContentType::Line(lines));
        }
    }

    /// Returns current clipboard content if present.
    pub fn get_content(&self) -> Option<&ClipboardContentType> {
        self.content.as_ref()
    }

    /// Clears internal clipboard.
    pub fn clear(&mut self) {
        self.content = None;
    }

    /// Returns true if clipboard is empty.
    pub fn is_empty(&self) -> bool {
        self.content.is_none()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_internal_clipboard_character_text() {
        let mut clipboard = InternalClipboard::new();
        assert!(clipboard.is_empty());

        clipboard.set_text("Copied String");
        assert!(!clipboard.is_empty());

        if let Some(ClipboardContentType::Character(s)) = clipboard.get_content() {
            assert_eq!(s, "Copied String");
        } else {
            panic!("Expected character content");
        }

        clipboard.clear();
        assert!(clipboard.is_empty());
    }

    #[test]
    fn test_internal_clipboard_line_text() {
        let mut clipboard = InternalClipboard::new();
        clipboard.set_lines(vec!["Line 1".into(), "Line 2".into()]);

        if let Some(ClipboardContentType::Line(lines)) = clipboard.get_content() {
            assert_eq!(lines.len(), 2);
            assert_eq!(lines[0], "Line 1");
        } else {
            panic!("Expected line content");
        }

        assert_eq!(
            clipboard.get_content().unwrap().as_str_content(),
            "Line 1\nLine 2"
        );
    }
}
