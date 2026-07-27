//! # Syntax Token Types
//!
//! Defines the semantic token categories used by the syntax highlighting
//! engine. Each token kind maps to a color in the active theme.

/// Semantic category of a syntax token.
///
/// These categories are language-agnostic — every programming language's
/// tokens are mapped into these categories by the highlighter. The theme
/// system then maps each category to a visual style.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TokenKind {
    /// Language keyword (e.g., `fn`, `let`, `if`, `for`).
    Keyword,
    /// String literal (including quotes).
    String,
    /// Numeric literal (integer, float, hex, etc.).
    Number,
    /// Comment (line or block).
    Comment,
    /// Operator (`+`, `-`, `=`, `=>`, etc.).
    Operator,
    /// Type name (e.g., `String`, `Vec`, `i32`).
    Type,
    /// Function name in a call or definition.
    Function,
    /// Constant value (e.g., `true`, `false`, `None`).
    Constant,
    /// Attribute or decorator (e.g., `#[derive]`, `@app.route`).
    Attribute,
    /// Variable or identifier.
    Variable,
    /// Error token (e.g., unterminated string).
    Error,
    /// Punctuation (`{`, `}`, `;`, `,`, etc.).
    Punctuation,
    /// Plain text that doesn't match any syntax rule.
    PlainText,
}

impl TokenKind {
    /// Returns a human-readable name for the token kind (useful for debugging).
    pub fn name(&self) -> &'static str {
        match self {
            Self::Keyword => "keyword",
            Self::String => "string",
            Self::Number => "number",
            Self::Comment => "comment",
            Self::Operator => "operator",
            Self::Type => "type",
            Self::Function => "function",
            Self::Constant => "constant",
            Self::Attribute => "attribute",
            Self::Variable => "variable",
            Self::Error => "error",
            Self::Punctuation => "punctuation",
            Self::PlainText => "plain_text",
        }
    }
}

/// A single highlighted token produced by the syntax engine.
///
/// Each token represents a contiguous span of text with a single semantic
/// category. The renderer uses the `kind` field to look up the appropriate
/// style from the active theme.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HighlightedToken {
    /// Semantic category of this token.
    pub kind: TokenKind,
    /// Start byte offset within the line.
    pub start: usize,
    /// End byte offset within the line (exclusive).
    pub end: usize,
    /// The text content of this token.
    pub text: String,
}

impl HighlightedToken {
    /// Creates a new highlighted token.
    pub fn new(kind: TokenKind, start: usize, end: usize, text: String) -> Self {
        Self {
            kind,
            start,
            end,
            text,
        }
    }

    /// Creates a plain text token.
    pub fn plain(start: usize, end: usize, text: String) -> Self {
        Self::new(TokenKind::PlainText, start, end, text)
    }

    /// Returns the byte length of this token.
    pub fn byte_len(&self) -> usize {
        self.end - self.start
    }

    /// Returns true if the token text is empty.
    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_kind_name() {
        assert_eq!(TokenKind::Keyword.name(), "keyword");
        assert_eq!(TokenKind::String.name(), "string");
        assert_eq!(TokenKind::Comment.name(), "comment");
        assert_eq!(TokenKind::PlainText.name(), "plain_text");
    }

    #[test]
    fn test_highlighted_token_creation() {
        let token = HighlightedToken::new(TokenKind::Keyword, 0, 2, "fn".to_string());
        assert_eq!(token.kind, TokenKind::Keyword);
        assert_eq!(token.start, 0);
        assert_eq!(token.end, 2);
        assert_eq!(token.text, "fn");
        assert_eq!(token.byte_len(), 2);
        assert!(!token.is_empty());
    }

    #[test]
    fn test_highlighted_token_plain() {
        let token = HighlightedToken::plain(5, 10, "hello".to_string());
        assert_eq!(token.kind, TokenKind::PlainText);
        assert_eq!(token.byte_len(), 5);
    }

    #[test]
    fn test_token_kind_equality() {
        assert_eq!(TokenKind::Keyword, TokenKind::Keyword);
        assert_ne!(TokenKind::Keyword, TokenKind::String);
    }
}
