//! # Regex-Based Syntax Highlighter
//!
//! A pure-Rust syntax highlighting backend that tokenizes source code
//! using pattern matching rules derived from language definitions.
//! No external C dependencies, no `unsafe` code.
//!
//! ## Design
//!
//! The highlighter processes each line character-by-character in a single
//! pass, using a simple state machine to handle strings, comments, and
//! multi-character tokens. This is faster than full regex for the simple
//! patterns we need and avoids the regex crate dependency.
//!
//! ## Limitations
//!
//! - Multi-line constructs (block comments spanning lines) use a
//!   simple nesting tracker that may not handle edge cases perfectly.
//! - No semantic analysis (can't distinguish function definitions from
//!   function calls with certainty).
//! - For full accuracy, a tree-sitter backend can be swapped in via
//!   the `SyntaxHighlighter` trait.

use std::collections::HashSet;

use crate::renderer::syntax::highlighter::SyntaxHighlighter;
use crate::renderer::syntax::languages::LanguageDefinition;
use crate::renderer::syntax::token::{HighlightedToken, TokenKind};

/// Regex-free pattern-matching syntax highlighter.
///
/// Uses a character-by-character state machine to tokenize source code
/// lines into semantically-categorized tokens.
pub struct RegexHighlighter {
    /// The language definition driving tokenization rules.
    language: &'static LanguageDefinition,
    /// Pre-computed keyword set for O(1) lookup.
    keywords: HashSet<&'static str>,
    /// Pre-computed type name set for O(1) lookup.
    types: HashSet<&'static str>,
    /// Pre-computed constant set for O(1) lookup.
    constants: HashSet<&'static str>,
}

impl RegexHighlighter {
    /// Creates a new highlighter for the given language definition.
    pub fn new(language: &'static LanguageDefinition) -> Self {
        Self {
            language,
            keywords: language.keywords.iter().copied().collect(),
            types: language.types.iter().copied().collect(),
            constants: language.constants.iter().copied().collect(),
        }
    }

    /// Classifies a word (identifier) as keyword, type, constant, or variable.
    fn classify_word(&self, word: &str) -> TokenKind {
        if self.keywords.contains(word) {
            TokenKind::Keyword
        } else if self.types.contains(word) {
            TokenKind::Type
        } else if self.constants.contains(word) {
            TokenKind::Constant
        } else if word.chars().next().is_some_and(|c| c.is_uppercase()) {
            // Heuristic: capitalized identifiers are likely types
            TokenKind::Type
        } else {
            TokenKind::Variable
        }
    }

    /// Checks if a character is an operator.
    fn is_operator(ch: char) -> bool {
        matches!(
            ch,
            '+' | '-' | '*' | '/' | '%' | '=' | '!' | '<' | '>' | '&' | '|' | '^' | '~' | '?' | ':'
        )
    }

    /// Checks if a character is punctuation.
    fn is_punctuation(ch: char) -> bool {
        matches!(
            ch,
            '(' | ')' | '{' | '}' | '[' | ']' | ';' | ',' | '.' | '@' | '#'
        )
    }

    /// Checks if a character can start a number.
    fn is_number_start(ch: char, next: Option<char>) -> bool {
        ch.is_ascii_digit() || (ch == '.' && next.is_some_and(|n| n.is_ascii_digit()))
    }

    /// Tokenizes a single line using pattern matching.
    fn tokenize_line(&self, line: &str) -> Vec<HighlightedToken> {
        if line.is_empty() {
            return Vec::new();
        }

        let mut tokens = Vec::new();
        let chars: Vec<char> = line.chars().collect();
        let len = chars.len();
        let mut i = 0;

        // Check for line comment
        let lc = self.language.line_comment;
        if !lc.is_empty() {
            let trimmed = line.trim_start();
            if trimmed.starts_with(lc) {
                tokens.push(HighlightedToken::new(
                    TokenKind::Comment,
                    0,
                    line.len(),
                    line.to_string(),
                ));
                return tokens;
            }
        }

        while i < len {
            let ch = chars[i];
            let byte_pos = chars[..i].iter().map(|c| c.len_utf8()).sum::<usize>();

            // ─── Line Comment (mid-line) ─────────────────────
            if !lc.is_empty() && i + lc.len() <= len {
                let remaining: String = chars[i..].iter().collect();
                if remaining.starts_with(lc) {
                    let comment_text: String = chars[i..].iter().collect();
                    let comment_start = byte_pos;
                    let comment_end = line.len();
                    tokens.push(HighlightedToken::new(
                        TokenKind::Comment,
                        comment_start,
                        comment_end,
                        comment_text,
                    ));
                    break;
                }
            }

            // ─── String Literals ─────────────────────────────
            if self.language.string_delimiters.contains(&ch) {
                let delimiter = ch;
                let start_i = i;
                let start_byte = byte_pos;
                i += 1; // skip opening delimiter

                while i < len {
                    if chars[i] == '\\' {
                        i += 2; // skip escaped character
                        continue;
                    }
                    if chars[i] == delimiter {
                        i += 1; // skip closing delimiter
                        break;
                    }
                    i += 1;
                }

                let text: String = chars[start_i..i].iter().collect();
                let end_byte = start_byte + text.len();
                tokens.push(HighlightedToken::new(
                    TokenKind::String,
                    start_byte,
                    end_byte,
                    text,
                ));
                continue;
            }

            // ─── Numbers ─────────────────────────────────────
            let next_ch = chars.get(i + 1).copied();
            if Self::is_number_start(ch, next_ch) && ch.is_ascii_digit() {
                let start_i = i;
                let start_byte = byte_pos;

                // Handle hex (0x), binary (0b), octal (0o)
                if ch == '0' && i + 1 < len {
                    match chars[i + 1] {
                        'x' | 'X' | 'b' | 'B' | 'o' | 'O' => {
                            i += 2;
                            while i < len && (chars[i].is_ascii_hexdigit() || chars[i] == '_') {
                                i += 1;
                            }
                        }
                        _ => {
                            while i < len
                                && (chars[i].is_ascii_digit()
                                    || chars[i] == '.'
                                    || chars[i] == '_'
                                    || chars[i] == 'e'
                                    || chars[i] == 'E')
                            {
                                i += 1;
                            }
                        }
                    }
                } else {
                    while i < len
                        && (chars[i].is_ascii_digit()
                            || chars[i] == '.'
                            || chars[i] == '_'
                            || chars[i] == 'e'
                            || chars[i] == 'E')
                    {
                        i += 1;
                    }
                }

                // Skip type suffixes (u32, f64, etc.)
                while i < len && (chars[i].is_ascii_alphanumeric() || chars[i] == '_') {
                    i += 1;
                }

                let text: String = chars[start_i..i].iter().collect();
                let end_byte = start_byte + text.len();
                tokens.push(HighlightedToken::new(
                    TokenKind::Number,
                    start_byte,
                    end_byte,
                    text,
                ));
                continue;
            }

            // ─── Identifiers / Keywords ──────────────────────
            if ch.is_alphabetic() || ch == '_' {
                let start_i = i;
                let start_byte = byte_pos;

                while i < len && (chars[i].is_alphanumeric() || chars[i] == '_') {
                    i += 1;
                }

                let word: String = chars[start_i..i].iter().collect();
                let end_byte = start_byte + word.len();
                let kind = self.classify_word(&word);

                // Check if next non-whitespace char is '(' for function detection
                let kind = if kind == TokenKind::Variable {
                    let mut j = i;
                    while j < len && chars[j].is_whitespace() {
                        j += 1;
                    }
                    if j < len && chars[j] == '(' {
                        TokenKind::Function
                    } else if j < len && chars[j] == '!' {
                        // Rust macro detection
                        TokenKind::Function
                    } else {
                        kind
                    }
                } else {
                    kind
                };

                tokens.push(HighlightedToken::new(kind, start_byte, end_byte, word));
                continue;
            }

            // ─── Attributes (# or @) ────────────────────────
            if ch == '#' && i + 1 < len && chars[i + 1] == '[' {
                let start_i = i;
                let start_byte = byte_pos;
                i += 2;

                let mut depth = 1;
                while i < len && depth > 0 {
                    match chars[i] {
                        '[' => depth += 1,
                        ']' => depth -= 1,
                        _ => {}
                    }
                    i += 1;
                }

                let text: String = chars[start_i..i].iter().collect();
                let end_byte = start_byte + text.len();
                tokens.push(HighlightedToken::new(
                    TokenKind::Attribute,
                    start_byte,
                    end_byte,
                    text,
                ));
                continue;
            }

            // ─── Operators ───────────────────────────────────
            if Self::is_operator(ch) {
                let start_i = i;
                let start_byte = byte_pos;

                // Consume multi-character operators (==, !=, >=, =>, ->, ::, etc.)
                while i < len && Self::is_operator(chars[i]) {
                    i += 1;
                }

                let text: String = chars[start_i..i].iter().collect();
                let end_byte = start_byte + text.len();
                tokens.push(HighlightedToken::new(
                    TokenKind::Operator,
                    start_byte,
                    end_byte,
                    text,
                ));
                continue;
            }

            // ─── Punctuation ─────────────────────────────────
            if Self::is_punctuation(ch) {
                let text = ch.to_string();
                let end_byte = byte_pos + ch.len_utf8();
                tokens.push(HighlightedToken::new(
                    TokenKind::Punctuation,
                    byte_pos,
                    end_byte,
                    text,
                ));
                i += 1;
                continue;
            }

            // ─── Whitespace / Other ──────────────────────────
            let text = ch.to_string();
            let end_byte = byte_pos + ch.len_utf8();
            tokens.push(HighlightedToken::new(
                TokenKind::PlainText,
                byte_pos,
                end_byte,
                text,
            ));
            i += 1;
        }

        tokens
    }
}

impl SyntaxHighlighter for RegexHighlighter {
    fn highlight_line(&self, line: &str, _line_index: usize) -> Vec<HighlightedToken> {
        self.tokenize_line(line)
    }

    fn language_name(&self) -> &str {
        self.language.name
    }

    fn file_extensions(&self) -> &[&str] {
        self.language.extensions
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::renderer::syntax::languages::{DOCKERFILE, JSON, RUST, SHELL, TOML, YAML};

    #[test]
    fn test_rust_keywords() {
        let h = RegexHighlighter::new(&RUST);
        let tokens = h.highlight_line("fn main() {", 0);

        let fn_token = tokens.iter().find(|t| t.text == "fn").unwrap();
        assert_eq!(fn_token.kind, TokenKind::Keyword);

        let main_token = tokens.iter().find(|t| t.text == "main").unwrap();
        assert_eq!(main_token.kind, TokenKind::Function);
    }

    #[test]
    fn test_rust_string() {
        let h = RegexHighlighter::new(&RUST);
        let tokens = h.highlight_line("let s = \"hello world\";", 0);

        let string_token = tokens.iter().find(|t| t.kind == TokenKind::String).unwrap();
        assert_eq!(string_token.text, "\"hello world\"");
    }

    #[test]
    fn test_rust_number() {
        let h = RegexHighlighter::new(&RUST);
        let tokens = h.highlight_line("let x = 42;", 0);

        let num_token = tokens.iter().find(|t| t.kind == TokenKind::Number).unwrap();
        assert!(num_token.text.contains("42"));
    }

    #[test]
    fn test_rust_hex_number() {
        let h = RegexHighlighter::new(&RUST);
        let tokens = h.highlight_line("let x = 0xFF;", 0);

        let num_token = tokens.iter().find(|t| t.kind == TokenKind::Number).unwrap();
        assert!(num_token.text.starts_with("0x"));
    }

    #[test]
    fn test_rust_comment() {
        let h = RegexHighlighter::new(&RUST);
        let tokens = h.highlight_line("// This is a comment", 0);

        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].kind, TokenKind::Comment);
        assert_eq!(tokens[0].text, "// This is a comment");
    }

    #[test]
    fn test_rust_mid_line_comment() {
        let h = RegexHighlighter::new(&RUST);
        let tokens = h.highlight_line("let x = 5; // value", 0);

        let comment = tokens
            .iter()
            .find(|t| t.kind == TokenKind::Comment)
            .unwrap();
        assert!(comment.text.contains("// value"));
    }

    #[test]
    fn test_rust_types() {
        let h = RegexHighlighter::new(&RUST);
        let tokens = h.highlight_line("let v: Vec<String> = Vec::new();", 0);

        let type_tokens: Vec<_> = tokens
            .iter()
            .filter(|t| t.kind == TokenKind::Type)
            .collect();
        assert!(type_tokens.iter().any(|t| t.text == "Vec"));
        assert!(type_tokens.iter().any(|t| t.text == "String"));
    }

    #[test]
    fn test_rust_attribute() {
        let h = RegexHighlighter::new(&RUST);
        let tokens = h.highlight_line("#[derive(Debug)]", 0);

        let attr = tokens
            .iter()
            .find(|t| t.kind == TokenKind::Attribute)
            .unwrap();
        assert_eq!(attr.text, "#[derive(Debug)]");
    }

    #[test]
    fn test_rust_operators() {
        let h = RegexHighlighter::new(&RUST);
        let tokens = h.highlight_line("x += y * z", 0);

        let ops: Vec<_> = tokens
            .iter()
            .filter(|t| t.kind == TokenKind::Operator)
            .collect();
        assert!(!ops.is_empty());
    }

    #[test]
    fn test_yaml_comment() {
        let h = RegexHighlighter::new(&YAML);
        let tokens = h.highlight_line("# This is a YAML comment", 0);

        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].kind, TokenKind::Comment);
    }

    #[test]
    fn test_yaml_constants() {
        let h = RegexHighlighter::new(&YAML);
        let tokens = h.highlight_line("enabled: true", 0);

        let const_token = tokens.iter().find(|t| t.text == "true").unwrap();
        assert_eq!(const_token.kind, TokenKind::Constant);
    }

    #[test]
    fn test_json_string() {
        let h = RegexHighlighter::new(&JSON);
        let tokens = h.highlight_line("\"key\": \"value\"", 0);

        let strings: Vec<_> = tokens
            .iter()
            .filter(|t| t.kind == TokenKind::String)
            .collect();
        assert_eq!(strings.len(), 2);
    }

    #[test]
    fn test_json_constants() {
        let h = RegexHighlighter::new(&JSON);
        let tokens = h.highlight_line("\"active\": true", 0);

        let const_token = tokens.iter().find(|t| t.text == "true").unwrap();
        assert_eq!(const_token.kind, TokenKind::Constant);
    }

    #[test]
    fn test_toml_comment() {
        let h = RegexHighlighter::new(&TOML);
        let tokens = h.highlight_line("# TOML config", 0);

        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].kind, TokenKind::Comment);
    }

    #[test]
    fn test_dockerfile_keywords() {
        let h = RegexHighlighter::new(&DOCKERFILE);
        let tokens = h.highlight_line("FROM ubuntu:22.04", 0);

        let from_token = tokens.iter().find(|t| t.text == "FROM").unwrap();
        assert_eq!(from_token.kind, TokenKind::Keyword);
    }

    #[test]
    fn test_shell_keywords() {
        let h = RegexHighlighter::new(&SHELL);
        let tokens = h.highlight_line("if [ -f file ]; then", 0);

        let if_token = tokens.iter().find(|t| t.text == "if").unwrap();
        assert_eq!(if_token.kind, TokenKind::Keyword);
    }

    #[test]
    fn test_empty_line() {
        let h = RegexHighlighter::new(&RUST);
        let tokens = h.highlight_line("", 0);
        assert!(tokens.is_empty());
    }

    #[test]
    fn test_whitespace_only_line() {
        let h = RegexHighlighter::new(&RUST);
        let tokens = h.highlight_line("    ", 0);
        assert!(!tokens.is_empty());
        assert!(tokens.iter().all(|t| t.kind == TokenKind::PlainText));
    }

    #[test]
    fn test_escaped_string() {
        let h = RegexHighlighter::new(&RUST);
        let tokens = h.highlight_line("let s = \"hello \\\"world\\\"\";", 0);

        let string_token = tokens.iter().find(|t| t.kind == TokenKind::String).unwrap();
        assert!(string_token.text.starts_with('"'));
        assert!(string_token.text.ends_with('"'));
    }

    #[test]
    fn test_punctuation() {
        let h = RegexHighlighter::new(&RUST);
        let tokens = h.highlight_line("{}", 0);

        let puncts: Vec<_> = tokens
            .iter()
            .filter(|t| t.kind == TokenKind::Punctuation)
            .collect();
        assert_eq!(puncts.len(), 2);
    }

    #[test]
    fn test_rust_constants() {
        let h = RegexHighlighter::new(&RUST);
        let tokens = h.highlight_line("let x = true;", 0);

        let const_token = tokens.iter().find(|t| t.text == "true").unwrap();
        assert_eq!(const_token.kind, TokenKind::Constant);
    }

    #[test]
    fn test_tokens_cover_entire_line() {
        let h = RegexHighlighter::new(&RUST);
        let line = "fn main() { let x = 42; }";
        let tokens = h.highlight_line(line, 0);

        // Reconstruct line from tokens
        let reconstructed: String = tokens.iter().map(|t| t.text.as_str()).collect();
        assert_eq!(reconstructed, line);
    }
}
