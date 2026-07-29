//! # Syntax Highlighting Engine
//!
//! Provides a trait-based syntax highlighting architecture that supports
//! multiple backends (regex-based, and future tree-sitter). The highlighting
//! engine tokenizes source code lines and maps tokens to semantic categories
//! that the theme system can style.
//!
//! ## Architecture
//!
//! ```text
//!  Source Line → SyntaxHighlighter::highlight_line() → Vec<HighlightedToken>
//!                                                           │
//!                                              Theme::style_for_token()
//!                                                           │
//!                                                    Styled ratatui Spans
//! ```
//!
//! ## Extension
//!
//! Adding a new language requires only adding a `LanguageDefinition` to the
//! language registry. Adding a new highlighting backend (e.g., tree-sitter)
//! requires implementing the `SyntaxHighlighter` trait — no existing code
//! needs to change.

pub mod highlighter;
pub mod languages;
pub mod regex_highlighter;
pub mod syntax_renderer;
pub mod token;
pub mod tree_sitter;

pub use highlighter::SyntaxHighlighter;
pub use languages::{LanguageDefinition, LanguageRegistry};
pub use regex_highlighter::RegexHighlighter;
pub use syntax_renderer::SyntaxRenderer;
pub use token::{HighlightedToken, TokenKind};
pub use tree_sitter::TreeSitterHighlighter;
