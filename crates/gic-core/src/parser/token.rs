//! Token classifications and token stream representations.

use crate::parser::position::TextRange;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Classification of lexical tokens.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum TokenKind {
    /// Language keyword (e.g. `if`, `resource`, `FROM`)
    Keyword,
    /// String literal (`"hello"`, `'value'`)
    String,
    /// Code comments (`# ...`, `// ...`, `/* ... */`)
    Comment,
    /// Operators (`=`, `==`, `+`, `-`, `=>`, `:`)
    Operator,
    /// Numeric values (`123`, `3.14`)
    Number,
    /// Function names or calls
    Function,
    /// Type declarations or annotations
    Type,
    /// Variables (`$VAR`, `${var.name}`)
    Variable,
    /// Identifiers / names
    Identifier,
    /// Whitespace (spaces, tabs, newlines)
    Whitespace,
    /// Punctuation (brackets, braces, commas, semicolons)
    Punctuation,
    /// Structural XML/HTML tag (`<tag>`)
    Tag,
    /// Key/Attribute name
    Attribute,
    /// Unrecognized or raw token
    Unknown,
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// Represents a single token extracted from source text.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Token {
    /// Category/Kind of the token.
    pub kind: TokenKind,
    /// Text range occupied by the token in the source document.
    pub range: TextRange,
    /// The exact substring text of the token.
    pub text: String,
}

impl Token {
    /// Creates a new `Token`.
    pub fn new(kind: TokenKind, range: TextRange, text: impl Into<String>) -> Self {
        Self {
            kind,
            range,
            text: text.into(),
        }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}({:?}, '{}')", self.range, self.kind, self.text)
    }
}

/// Helper container for token streams.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TokenStream {
    pub tokens: Vec<Token>,
}

impl TokenStream {
    /// Creates an empty token stream.
    pub fn new() -> Self {
        Self { tokens: Vec::new() }
    }

    /// Adds a token to the stream.
    pub fn push(&mut self, token: Token) {
        self.tokens.push(token);
    }

    /// Returns tokens matching non-whitespace kinds.
    pub fn non_whitespace(&self) -> Vec<&Token> {
        self.tokens
            .iter()
            .filter(|t| t.kind != TokenKind::Whitespace)
            .collect()
    }
}
