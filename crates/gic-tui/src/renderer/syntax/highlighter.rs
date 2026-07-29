//! # Syntax Highlighter Trait
//!
//! Defines the interface that all syntax highlighting backends must implement.
//! This trait enables swapping between regex-based highlighting and tree-sitter
//! (or any other backend) without changing the rendering pipeline.

use crate::renderer::syntax::token::HighlightedToken;

/// Trait for syntax highlighting backends.
///
/// Implementors tokenize source code lines into sequences of
/// [`HighlightedToken`] values that the renderer can style.
///
/// # Implementation Requirements
///
/// - `highlight_line` must never panic — return a single `PlainText` token
///   on any error.
/// - Tokens must cover the entire line text with no gaps or overlaps.
/// - Token `start`/`end` byte offsets must be valid indices into the line.
///
/// # Thread Safety
///
/// Highlighters are not required to be `Send` or `Sync`. They are used
/// within a single rendering thread.
pub trait SyntaxHighlighter {
    /// Tokenizes a single line of source code.
    ///
    /// # Arguments
    ///
    /// * `line` - The text content of the line to highlight.
    /// * `line_index` - The 0-indexed line number (useful for multi-line
    ///   context such as block comments).
    ///
    /// # Returns
    ///
    /// A vector of [`HighlightedToken`]s covering the entire line.
    fn highlight_line(&self, line: &str, line_index: usize) -> Vec<HighlightedToken>;

    /// Updates the highlighter with the full text buffer.
    ///
    /// This is used by stateful highlighters (e.g. tree-sitter) that need
    /// to parse the entire document to provide accurate highlighting.
    ///
    /// The default implementation does nothing.
    fn update_buffer(&self, _text: &str) {}

    /// Returns the human-readable name of the language this highlighter supports.
    fn language_name(&self) -> &str;

    /// Returns the file extensions this highlighter handles (without leading dot).
    ///
    /// # Examples
    ///
    /// ```text
    /// ["rs"]           // Rust
    /// ["yml", "yaml"]  // YAML
    /// ["tf", "tfvars"] // Terraform
    /// ```
    fn file_extensions(&self) -> &[&str];
}

/// A no-op highlighter that returns the entire line as plain text.
///
/// Used as a fallback when no language-specific highlighter is available.
pub struct PlainTextHighlighter;

impl SyntaxHighlighter for PlainTextHighlighter {
    fn highlight_line(&self, line: &str, _line_index: usize) -> Vec<HighlightedToken> {
        if line.is_empty() {
            return Vec::new();
        }
        vec![HighlightedToken::plain(0, line.len(), line.to_string())]
    }

    fn language_name(&self) -> &str {
        "Plain Text"
    }

    fn file_extensions(&self) -> &[&str] {
        &["txt", "text", "log"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::renderer::syntax::token::TokenKind;

    #[test]
    fn test_plain_text_highlighter() {
        let h = PlainTextHighlighter;
        let tokens = h.highlight_line("Hello World", 0);

        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].kind, TokenKind::PlainText);
        assert_eq!(tokens[0].text, "Hello World");
        assert_eq!(tokens[0].start, 0);
        assert_eq!(tokens[0].end, 11);
    }

    #[test]
    fn test_plain_text_empty_line() {
        let h = PlainTextHighlighter;
        let tokens = h.highlight_line("", 0);
        assert!(tokens.is_empty());
    }

    #[test]
    fn test_plain_text_language_info() {
        let h = PlainTextHighlighter;
        assert_eq!(h.language_name(), "Plain Text");
        assert!(h.file_extensions().contains(&"txt"));
    }
}
