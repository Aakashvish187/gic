use crate::renderer::syntax::highlighter::SyntaxHighlighter;
use crate::renderer::syntax::token::{HighlightedToken, TokenKind};
use std::cell::RefCell;
use std::collections::HashMap;
use streaming_iterator::StreamingIterator;

/// A stateful Tree-sitter highlighter that parses the full document and caches highlights per line.
pub struct TreeSitterHighlighter {
    language_name: &'static str,
    extensions: &'static [&'static str],
    parser: RefCell<tree_sitter::Parser>,
    query: tree_sitter::Query,
    /// Cached tokens per line index.
    cache: RefCell<HashMap<usize, Vec<HighlightedToken>>>,
}

impl TreeSitterHighlighter {
    /// Creates a new TreeSitterHighlighter for a given language and query string.
    pub fn new(
        language_name: &'static str,
        extensions: &'static [&'static str],
        language: tree_sitter::Language,
        query_string: &str,
    ) -> Result<Self, tree_sitter::QueryError> {
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&language)
            .expect("Failed to set language");

        let query = tree_sitter::Query::new(&language, query_string)?;

        Ok(Self {
            language_name,
            extensions,
            parser: RefCell::new(parser),
            query,
            cache: RefCell::new(HashMap::new()),
        })
    }

    /// Maps a tree-sitter capture name to our TokenKind.
    fn map_capture_to_token_kind(capture_name: &str) -> TokenKind {
        match capture_name {
            "keyword" | "keyword.function" | "keyword.return" | "keyword.operator"
            | "keyword.control" => TokenKind::Keyword,
            "type" | "type.builtin" | "type.qualifier" => TokenKind::Type,
            "function" | "function.builtin" | "function.macro" => TokenKind::Function,
            "string" | "string.escape" | "string.special" => TokenKind::String,
            "number" | "number.float" | "boolean" | "constant.builtin" => TokenKind::Constant,
            "comment" | "comment.line" | "comment.block" => TokenKind::Comment,
            "variable" | "variable.parameter" => TokenKind::Variable,
            "property" | "attribute" => TokenKind::Variable,
            "punctuation.bracket" | "punctuation.delimiter" => TokenKind::Operator,
            "operator" => TokenKind::Operator,
            _ => TokenKind::PlainText,
        }
    }
}

impl SyntaxHighlighter for TreeSitterHighlighter {
    fn highlight_line(&self, line: &str, line_index: usize) -> Vec<HighlightedToken> {
        let cache = self.cache.borrow();
        if let Some(tokens) = cache.get(&line_index) {
            return tokens.clone();
        }

        // Fallback if update_buffer wasn't called or line was out of bounds
        vec![HighlightedToken::plain(0, line.len(), line.to_string())]
    }

    fn update_buffer(&self, text: &str) {
        let mut parser = self.parser.borrow_mut();
        // Parse the full buffer
        if let Some(tree) = parser.parse(text, None) {
            let mut cursor = tree_sitter::QueryCursor::new();
            let mut matches = cursor.matches(&self.query, tree.root_node(), text.as_bytes());

            let mut line_tokens: HashMap<usize, Vec<HighlightedToken>> = HashMap::new();

            while let Some(m) = matches.next() {
                for capture in m.captures {
                    let capture_name: &str = &self.query.capture_names()[capture.index as usize];
                    let token_kind = Self::map_capture_to_token_kind(capture_name);

                    let start_pos = capture.node.start_position();
                    let end_pos = capture.node.end_position();

                    // Multi-line tokens are rare in standard syntax (except strings/comments),
                    // but we must handle them line by line.
                    for row in start_pos.row..=end_pos.row {
                        let mut start_byte_in_line = 0;
                        let mut end_byte_in_line = 0; // Will be populated with line len later if needed

                        // Note: We need byte offsets relative to the line. Tree-sitter provides column which is a byte offset in the line!
                        if row == start_pos.row {
                            start_byte_in_line = start_pos.column;
                        }

                        let mut end_col = usize::MAX;
                        if row == end_pos.row {
                            end_col = end_pos.column;
                        }

                        // We will just store raw highlight requests.
                        // We extract the actual substring later or we could do it now if we iterate lines.
                        let line_str = text.lines().nth(row).unwrap_or("");
                        if end_col == usize::MAX {
                            end_col = line_str.len();
                        }

                        // Clamp
                        start_byte_in_line = start_byte_in_line.min(line_str.len());
                        end_col = end_col.min(line_str.len());

                        if start_byte_in_line < end_col {
                            let token_text = line_str[start_byte_in_line..end_col].to_string();
                            let token = HighlightedToken::new(
                                token_kind,
                                start_byte_in_line,
                                end_col,
                                token_text,
                            );
                            line_tokens.entry(row).or_default().push(token);
                        }
                    }
                }
            }

            // Fill gaps with PlainText
            for (row, tokens) in line_tokens.iter_mut() {
                tokens.sort_by_key(|t| t.start);

                let line_str = text.lines().nth(*row).unwrap_or("");
                let mut full_tokens = Vec::new();
                let mut current_offset = 0;

                for token in tokens.iter() {
                    // Ignore overlapping tokens for simplicity
                    if token.start < current_offset {
                        continue;
                    }

                    if token.start > current_offset {
                        let text = line_str[current_offset..token.start].to_string();
                        full_tokens.push(HighlightedToken::plain(
                            current_offset,
                            token.start,
                            text,
                        ));
                    }
                    full_tokens.push(token.clone());
                    current_offset = token.end;
                }

                if current_offset < line_str.len() {
                    let text = line_str[current_offset..line_str.len()].to_string();
                    full_tokens.push(HighlightedToken::plain(
                        current_offset,
                        line_str.len(),
                        text,
                    ));
                }

                *tokens = full_tokens;
            }

            // Update cache
            *self.cache.borrow_mut() = line_tokens;
        }
    }

    fn language_name(&self) -> &str {
        self.language_name
    }

    fn file_extensions(&self) -> &[&str] {
        self.extensions
    }
}
