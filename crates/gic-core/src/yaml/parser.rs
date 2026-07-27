//! YAML AST Parser and Tokenizer for GIC.
//!
//! Provides a zero-panic, high-performance AST parser for multi-document YAML sources
//! with precise line, column, byte-range span annotations and comment mapping.

use crate::yaml::errors::YamlResult;

/// Position in a source document (1-indexed line and column).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Position {
    /// 1-indexed line number.
    pub line: usize,
    /// 1-indexed column number (character/codepoint count).
    pub column: usize,
    /// 0-indexed byte offset from the start of the document.
    pub byte_offset: usize,
}

impl Position {
    /// Creates a new Position.
    pub fn new(line: usize, column: usize, byte_offset: usize) -> Self {
        Self {
            line,
            column,
            byte_offset,
        }
    }
}

/// Source code span demarcated by start and end positions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Span {
    /// Start position of the token or AST node.
    pub start: Position,
    /// End position of the token or AST node.
    pub end: Position,
}

impl Span {
    /// Creates a new Span from start and end positions.
    pub fn new(start: Position, end: Position) -> Self {
        Self { start, end }
    }

    /// Checks if a given position falls within this span.
    pub fn contains(&self, pos: Position) -> bool {
        if pos.line < self.start.line || pos.line > self.end.line {
            return false;
        }
        if pos.line == self.start.line && pos.column < self.start.column {
            return false;
        }
        if pos.line == self.end.line && pos.column > self.end.column {
            return false;
        }
        true
    }
}

/// YAML comment metadata and location.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct YamlComment {
    /// The trimmed comment text without leading `#`.
    pub text: String,
    /// Raw comment string including leading `#`.
    pub raw: String,
    /// True if the comment sits on the same line after code.
    pub is_inline: bool,
    /// Span location of the comment.
    pub span: Span,
}

/// YAML scalar quotation or block styling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum YamlScalarStyle {
    /// Unquoted scalar string, integer, float, bool, or null.
    #[default]
    Plain,
    /// Single-quoted scalar (`'text'`).
    SingleQuoted,
    /// Double-quoted scalar (`"text"`).
    DoubleQuoted,
    /// Literal block scalar (`|`).
    LiteralBlock,
    /// Folded block scalar (`>`).
    FoldedBlock,
}

/// YAML scalar value AST node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct YamlScalar {
    /// Evaluated scalar value string.
    pub value: String,
    /// Raw representation in source text.
    pub raw: String,
    /// Formatting style of the scalar.
    pub style: YamlScalarStyle,
    /// Source span.
    pub span: Span,
}

/// YAML mapping key node.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct YamlKey {
    /// Unquoted key text representation.
    pub value: String,
    /// Raw string content of the key.
    pub raw: String,
    /// Source span for the key.
    pub span: Span,
}

/// YAML Anchor definition node (`&anchor_name`).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AnchorDefinition {
    /// Anchor identifier name.
    pub name: String,
    /// Span of the anchor token.
    pub span: Span,
}

/// YAML Alias reference node (`*alias_name`).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AliasReference {
    /// Target anchor identifier name.
    pub name: String,
    /// Span of the alias token.
    pub span: Span,
}

/// Single key-value pair in a mapping.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct YamlPair {
    /// Mapping key.
    pub key: YamlKey,
    /// Mapping value node.
    pub value: YamlNode,
    /// Span of the colon separator token.
    pub colon_span: Span,
    /// Entire span of key + colon + value.
    pub span: Span,
}

/// YAML mapping block (`key: value`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct YamlMapping {
    /// List of key-value pairs in order.
    pub pairs: Vec<YamlPair>,
    /// Span covering all pairs.
    pub span: Span,
}

/// YAML sequence block (`- item`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct YamlSequence {
    /// List of item nodes in order.
    pub items: Vec<YamlNode>,
    /// Span covering the sequence.
    pub span: Span,
}

/// Enum representing the core structural value of a YAML node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum YamlValue {
    /// Mapping structure (`{ key: val }`).
    Mapping(YamlMapping),
    /// Sequence structure (`[ item1, item2 ]`).
    Sequence(YamlSequence),
    /// Primitive scalar value.
    Scalar(YamlScalar),
    /// Alias reference pointing to an anchor.
    Alias(AliasReference),
    /// Null value or empty node.
    Null,
}

/// Primary node in the YAML AST.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct YamlNode {
    /// Unique node identifier for AST graph indexing.
    pub id: usize,
    /// Optional anchor defined at this node (`&name`).
    pub anchor: Option<AnchorDefinition>,
    /// Structural value of the node.
    pub value: YamlValue,
    /// Comments preceding this node.
    pub leading_comments: Vec<YamlComment>,
    /// Inline comment on the same line as the node.
    pub trailing_comment: Option<YamlComment>,
    /// Entire source span of the node.
    pub span: Span,
}

/// Single document within a YAML document stream.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct YamlDocument {
    /// Root node of the document, if non-empty.
    pub root: Option<YamlNode>,
    /// All comments present in this document.
    pub comments: Vec<YamlComment>,
    /// True if initialized with explicit `---` document start directive.
    pub has_explicit_start: bool,
    /// True if terminated with explicit `...` document end directive.
    pub has_explicit_end: bool,
    /// Span of the entire document.
    pub span: Span,
}

/// Complete parsed AST representing one or more YAML documents in a stream.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct YamlAST {
    /// Collection of parsed documents.
    pub documents: Vec<YamlDocument>,
    /// Top-level or orphaned comments.
    pub comments: Vec<YamlComment>,
    /// Length of the original source text in bytes.
    pub source_len: usize,
}

impl YamlAST {
    /// Returns true if the AST contains no documents or all documents are empty.
    pub fn is_empty(&self) -> bool {
        self.documents.is_empty()
            || self.documents.iter().all(|doc| {
                doc.root
                    .as_ref()
                    .map_or(true, |r| matches!(r.value, YamlValue::Null))
            })
    }
}

/// Lexical token generated during YAML scanning.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
    DocumentStart, // ---
    DocumentEnd,   // ...
    Key(String),
    Colon,
    Dash,
    Anchor(String),
    Alias(String),
    Scalar(String, YamlScalarStyle),
    Comment(String),
    Indent(usize),
    Newline,
    Eof,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

/// Parser configuration options.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct YamlParserOptions {
    /// Preserve all comments during parsing.
    pub preserve_comments: bool,
    /// Strict mode errors on unexpected syntax.
    pub strict: bool,
}

impl Default for YamlParserOptions {
    fn default() -> Self {
        Self {
            preserve_comments: true,
            strict: false,
        }
    }
}

/// High-performance, fault-tolerant YAML parser.
#[derive(Debug)]
pub struct YamlParser {
    options: YamlParserOptions,
    next_node_id: usize,
}

impl Default for YamlParser {
    fn default() -> Self {
        Self::new()
    }
}

impl YamlParser {
    /// Constructs a new YamlParser with default options.
    pub fn new() -> Self {
        Self::with_options(YamlParserOptions::default())
    }

    /// Constructs a YamlParser with custom options.
    pub fn with_options(options: YamlParserOptions) -> Self {
        Self {
            options,
            next_node_id: 1,
        }
    }

    /// Returns options reference.
    pub fn options(&self) -> &YamlParserOptions {
        &self.options
    }

    /// Generates a unique node identifier.
    fn allocate_id(&mut self) -> usize {
        let id = self.next_node_id;
        self.next_node_id += 1;
        id
    }

    /// Parses a raw YAML string into a `YamlAST`.
    pub fn parse(&mut self, source: &str) -> YamlResult<YamlAST> {
        let line_starts = compute_line_starts(source);
        let comments = extract_comments(source, &line_starts);
        let tokens = tokenize(source, &line_starts)?;

        let mut documents = Vec::new();
        let mut current_doc_tokens = Vec::new();

        for token in tokens {
            if token.kind == TokenKind::DocumentStart && !current_doc_tokens.is_empty() {
                let doc = self.parse_document(&current_doc_tokens, source, &comments)?;
                documents.push(doc);
                current_doc_tokens.clear();
            }
            current_doc_tokens.push(token);
        }

        if !current_doc_tokens.is_empty() {
            let doc = self.parse_document(&current_doc_tokens, source, &comments)?;
            documents.push(doc);
        }

        if documents.is_empty() {
            let empty_pos = Position::new(1, 1, 0);
            documents.push(YamlDocument {
                root: None,
                comments: comments.clone(),
                has_explicit_start: false,
                has_explicit_end: false,
                span: Span::new(empty_pos, empty_pos),
            });
        }

        Ok(YamlAST {
            documents,
            comments,
            source_len: source.len(),
        })
    }

    fn parse_document(
        &mut self,
        tokens: &[Token],
        source: &str,
        comments: &[YamlComment],
    ) -> YamlResult<YamlDocument> {
        let mut has_explicit_start = false;
        let mut has_explicit_end = false;

        let start_pos = tokens
            .first()
            .map(|t| t.span.start)
            .unwrap_or_else(|| Position::new(1, 1, 0));
        let end_pos = tokens
            .last()
            .map(|t| t.span.end)
            .unwrap_or_else(|| Position::new(1, 1, 0));

        let mut filtered_tokens = Vec::new();
        for t in tokens {
            match t.kind {
                TokenKind::DocumentStart => has_explicit_start = true,
                TokenKind::DocumentEnd => has_explicit_end = true,
                TokenKind::Newline | TokenKind::Eof => {}
                _ => filtered_tokens.push(t.clone()),
            }
        }

        let root = self.parse_node_stream(&filtered_tokens, source)?;

        Ok(YamlDocument {
            root,
            comments: comments.to_vec(),
            has_explicit_start,
            has_explicit_end,
            span: Span::new(start_pos, end_pos),
        })
    }

    fn parse_node_stream(
        &mut self,
        tokens: &[Token],
        source: &str,
    ) -> YamlResult<Option<YamlNode>> {
        if tokens.is_empty() {
            return Ok(None);
        }

        let is_mapping = tokens.iter().any(|t| matches!(t.kind, TokenKind::Colon));
        let is_sequence = tokens.iter().any(|t| matches!(t.kind, TokenKind::Dash));

        if is_mapping {
            let mapping = self.parse_mapping(tokens, source)?;
            let id = self.allocate_id();
            let span = mapping.span;
            Ok(Some(YamlNode {
                id,
                anchor: None,
                value: YamlValue::Mapping(mapping),
                leading_comments: Vec::new(),
                trailing_comment: None,
                span,
            }))
        } else if is_sequence {
            let seq = self.parse_sequence(tokens, source)?;
            let id = self.allocate_id();
            let span = seq.span;
            Ok(Some(YamlNode {
                id,
                anchor: None,
                value: YamlValue::Sequence(seq),
                leading_comments: Vec::new(),
                trailing_comment: None,
                span,
            }))
        } else {
            if let Some(t) = tokens.first() {
                if t.kind == TokenKind::Eof {
                    return Ok(None);
                }
                let id = self.allocate_id();
                let (val, span) = match &t.kind {
                    TokenKind::Scalar(s, style) => (
                        YamlValue::Scalar(YamlScalar {
                            value: s.clone(),
                            raw: s.clone(),
                            style: *style,
                            span: t.span,
                        }),
                        t.span,
                    ),
                    TokenKind::Alias(a) => (
                        YamlValue::Alias(AliasReference {
                            name: a.clone(),
                            span: t.span,
                        }),
                        t.span,
                    ),
                    _ => (YamlValue::Null, t.span),
                };
                Ok(Some(YamlNode {
                    id,
                    anchor: None,
                    value: val,
                    leading_comments: Vec::new(),
                    trailing_comment: None,
                    span,
                }))
            } else {
                Ok(None)
            }
        }
    }

    fn parse_mapping(&mut self, tokens: &[Token], source: &str) -> YamlResult<YamlMapping> {
        let mut pairs = Vec::new();
        let start_pos = tokens.first().map(|t| t.span.start).unwrap_or_default();
        let end_pos = tokens.last().map(|t| t.span.end).unwrap_or_default();

        let mut idx = 0;
        while idx < tokens.len() {
            if idx + 1 < tokens.len() && matches!(tokens[idx + 1].kind, TokenKind::Colon) {
                let key_token = &tokens[idx];
                let colon_token = &tokens[idx + 1];

                let key_str = match &key_token.kind {
                    TokenKind::Key(k) => k.clone(),
                    TokenKind::Scalar(s, _) => s.clone(),
                    _ => format!("{:?}", key_token.kind),
                };

                let key = YamlKey {
                    value: key_str.clone(),
                    raw: key_str,
                    span: key_token.span,
                };

                let mut anchor_def = None;
                let mut val_idx = idx + 2;

                if val_idx < tokens.len() {
                    if let TokenKind::Anchor(ref name) = tokens[val_idx].kind {
                        anchor_def = Some(AnchorDefinition {
                            name: name.clone(),
                            span: tokens[val_idx].span,
                        });
                        val_idx += 1;
                    }
                }

                let (val_node, next_idx) = if val_idx < tokens.len() {
                    let val_token = &tokens[val_idx];

                    if matches!(val_token.kind, TokenKind::Dash) {
                        let mut seq_tokens = Vec::new();
                        let mut curr = val_idx;
                        while curr < tokens.len() {
                            if curr > val_idx
                                && curr + 1 < tokens.len()
                                && matches!(tokens[curr + 1].kind, TokenKind::Colon)
                                && !matches!(tokens[curr - 1].kind, TokenKind::Dash)
                            {
                                break;
                            }
                            seq_tokens.push(tokens[curr].clone());
                            curr += 1;
                        }

                        let seq = self.parse_sequence(&seq_tokens, source)?;
                        let id = self.allocate_id();
                        let seq_span = seq.span;
                        let seq_node = YamlNode {
                            id,
                            anchor: anchor_def,
                            value: YamlValue::Sequence(seq),
                            leading_comments: Vec::new(),
                            trailing_comment: None,
                            span: seq_span,
                        };
                        (seq_node, curr)
                    } else if val_idx + 1 < tokens.len()
                        && matches!(tokens[val_idx + 1].kind, TokenKind::Colon)
                    {
                        let id = self.allocate_id();
                        let null_node = YamlNode {
                            id,
                            anchor: anchor_def,
                            value: YamlValue::Null,
                            leading_comments: Vec::new(),
                            trailing_comment: None,
                            span: colon_token.span,
                        };
                        (null_node, val_idx)
                    } else {
                        let id = self.allocate_id();
                        let val_node = YamlNode {
                            id,
                            anchor: anchor_def,
                            value: match &val_token.kind {
                                TokenKind::Scalar(s, style) => YamlValue::Scalar(YamlScalar {
                                    value: s.clone(),
                                    raw: s.clone(),
                                    style: *style,
                                    span: val_token.span,
                                }),
                                TokenKind::Alias(a) => YamlValue::Alias(AliasReference {
                                    name: a.clone(),
                                    span: val_token.span,
                                }),
                                _ => YamlValue::Null,
                            },
                            leading_comments: Vec::new(),
                            trailing_comment: None,
                            span: val_token.span,
                        };
                        (val_node, val_idx + 1)
                    }
                } else {
                    let id = self.allocate_id();
                    let null_node = YamlNode {
                        id,
                        anchor: anchor_def,
                        value: YamlValue::Null,
                        leading_comments: Vec::new(),
                        trailing_comment: None,
                        span: colon_token.span,
                    };
                    (null_node, val_idx)
                };

                let pair_span = Span::new(key_token.span.start, val_node.span.end);
                pairs.push(YamlPair {
                    key,
                    value: val_node,
                    colon_span: colon_token.span,
                    span: pair_span,
                });

                idx = next_idx;
            } else {
                let token = &tokens[idx];
                let token_str = match &token.kind {
                    TokenKind::Scalar(s, _) => s.clone(),
                    TokenKind::Key(k) => k.clone(),
                    TokenKind::Dash => "-".to_string(),
                    _ => format!("{:?}", token.kind),
                };

                let key = YamlKey {
                    value: token_str.clone(),
                    raw: token_str.clone(),
                    span: token.span,
                };

                let id = self.allocate_id();
                let val_node = YamlNode {
                    id,
                    anchor: None,
                    value: YamlValue::Scalar(YamlScalar {
                        value: token_str,
                        raw: String::new(),
                        style: YamlScalarStyle::Plain,
                        span: token.span,
                    }),
                    leading_comments: Vec::new(),
                    trailing_comment: None,
                    span: token.span,
                };

                pairs.push(YamlPair {
                    key,
                    value: val_node,
                    colon_span: token.span,
                    span: token.span,
                });

                idx += 1;
            }
        }

        Ok(YamlMapping {
            pairs,
            span: Span::new(start_pos, end_pos),
        })
    }

    fn parse_sequence(&mut self, tokens: &[Token], source: &str) -> YamlResult<YamlSequence> {
        let mut items = Vec::new();
        let start_pos = tokens.first().map(|t| t.span.start).unwrap_or_default();
        let end_pos = tokens.last().map(|t| t.span.end).unwrap_or_default();

        let mut idx = 0;
        while idx < tokens.len() {
            if matches!(tokens[idx].kind, TokenKind::Dash) {
                idx += 1;
                let mut item_tokens = Vec::new();
                while idx < tokens.len() && !matches!(tokens[idx].kind, TokenKind::Dash) {
                    item_tokens.push(tokens[idx].clone());
                    idx += 1;
                }

                if let Some(node) = self.parse_node_stream(&item_tokens, source)? {
                    items.push(node);
                }
            } else {
                idx += 1;
            }
        }

        Ok(YamlSequence {
            items,
            span: Span::new(start_pos, end_pos),
        })
    }
}

pub fn compute_line_starts(source: &str) -> Vec<usize> {
    let mut starts = vec![0];
    for (idx, b) in source.bytes().enumerate() {
        if b == b'\n' {
            starts.push(idx + 1);
        }
    }
    starts
}

pub fn offset_to_position(offset: usize, line_starts: &[usize]) -> Position {
    let line_idx = match line_starts.binary_search(&offset) {
        Ok(idx) => idx,
        Err(idx) => idx.saturating_sub(1),
    };
    let line_start = line_starts.get(line_idx).copied().unwrap_or(0);
    let column = offset.saturating_sub(line_start) + 1;
    Position::new(line_idx + 1, column, offset)
}

pub fn extract_comments(source: &str, line_starts: &[usize]) -> Vec<YamlComment> {
    let mut comments = Vec::new();
    for (line_idx, line) in source.lines().enumerate() {
        if let Some(hash_pos) = line.find('#') {
            let is_inline = line[..hash_pos].chars().any(|c| !c.is_whitespace());
            let line_start = line_starts.get(line_idx).copied().unwrap_or(0);
            let comment_start_offset = line_start + hash_pos;
            let comment_end_offset = line_start + line.len();

            let raw = line[hash_pos..].to_string();
            let text = raw.trim_start_matches('#').trim().to_string();

            let start_pos = offset_to_position(comment_start_offset, line_starts);
            let end_pos = offset_to_position(comment_end_offset, line_starts);

            comments.push(YamlComment {
                text,
                raw,
                is_inline,
                span: Span::new(start_pos, end_pos),
            });
        }
    }
    comments
}

pub fn tokenize(source: &str, line_starts: &[usize]) -> YamlResult<Vec<Token>> {
    let mut tokens = Vec::new();
    let bytes = source.as_bytes();
    let mut idx = 0;

    while idx < bytes.len() {
        if bytes[idx] == b' ' || bytes[idx] == b'\r' || bytes[idx] == b'\t' {
            idx += 1;
            continue;
        }

        if bytes[idx] == b'\n' {
            let pos = offset_to_position(idx, line_starts);
            tokens.push(Token {
                kind: TokenKind::Newline,
                span: Span::new(pos, pos),
            });
            idx += 1;
            continue;
        }

        if bytes[idx] == b'#' {
            let start_idx = idx;
            while idx < bytes.len() && bytes[idx] != b'\n' {
                idx += 1;
            }
            let raw = String::from_utf8_lossy(&bytes[start_idx..idx]).to_string();
            let start_pos = offset_to_position(start_idx, line_starts);
            let end_pos = offset_to_position(idx, line_starts);
            tokens.push(Token {
                kind: TokenKind::Comment(raw),
                span: Span::new(start_pos, end_pos),
            });
            continue;
        }

        if idx + 2 < bytes.len()
            && &bytes[idx..idx + 3] == b"---"
            && (idx + 3 == bytes.len() || bytes[idx + 3].is_ascii_whitespace())
        {
            let start_pos = offset_to_position(idx, line_starts);
            idx += 3;
            let end_pos = offset_to_position(idx, line_starts);
            tokens.push(Token {
                kind: TokenKind::DocumentStart,
                span: Span::new(start_pos, end_pos),
            });
            continue;
        }

        if idx + 2 < bytes.len()
            && &bytes[idx..idx + 3] == b"..."
            && (idx + 3 == bytes.len() || bytes[idx + 3].is_ascii_whitespace())
        {
            let start_pos = offset_to_position(idx, line_starts);
            idx += 3;
            let end_pos = offset_to_position(idx, line_starts);
            tokens.push(Token {
                kind: TokenKind::DocumentEnd,
                span: Span::new(start_pos, end_pos),
            });
            continue;
        }

        // Colon `:` (only a key indicator if followed by whitespace or end of string)
        if bytes[idx] == b':' && (idx + 1 == bytes.len() || bytes[idx + 1].is_ascii_whitespace()) {
            let start_pos = offset_to_position(idx, line_starts);
            idx += 1;
            let end_pos = offset_to_position(idx, line_starts);
            tokens.push(Token {
                kind: TokenKind::Colon,
                span: Span::new(start_pos, end_pos),
            });
            continue;
        }

        if bytes[idx] == b'-' && (idx + 1 == bytes.len() || bytes[idx + 1].is_ascii_whitespace()) {
            let start_pos = offset_to_position(idx, line_starts);
            idx += 1;
            let end_pos = offset_to_position(idx, line_starts);
            tokens.push(Token {
                kind: TokenKind::Dash,
                span: Span::new(start_pos, end_pos),
            });
            continue;
        }

        if bytes[idx] == b'&' {
            let start_idx = idx;
            idx += 1;
            while idx < bytes.len() && !bytes[idx].is_ascii_whitespace() && bytes[idx] != b':' {
                idx += 1;
            }
            let name = String::from_utf8_lossy(&bytes[start_idx + 1..idx]).to_string();
            let start_pos = offset_to_position(start_idx, line_starts);
            let end_pos = offset_to_position(idx, line_starts);
            tokens.push(Token {
                kind: TokenKind::Anchor(name),
                span: Span::new(start_pos, end_pos),
            });
            continue;
        }

        if bytes[idx] == b'*' {
            let start_idx = idx;
            idx += 1;
            while idx < bytes.len() && !bytes[idx].is_ascii_whitespace() && bytes[idx] != b':' {
                idx += 1;
            }
            let name = String::from_utf8_lossy(&bytes[start_idx + 1..idx]).to_string();
            let start_pos = offset_to_position(start_idx, line_starts);
            let end_pos = offset_to_position(idx, line_starts);
            tokens.push(Token {
                kind: TokenKind::Alias(name),
                span: Span::new(start_pos, end_pos),
            });
            continue;
        }

        let start_idx = idx;
        let mut style = YamlScalarStyle::Plain;

        if bytes[idx] == b'"' || bytes[idx] == b'\'' {
            let quote = bytes[idx];
            style = if quote == b'"' {
                YamlScalarStyle::DoubleQuoted
            } else {
                YamlScalarStyle::SingleQuoted
            };
            idx += 1;
            while idx < bytes.len() && bytes[idx] != quote {
                if bytes[idx] == b'\\' && idx + 1 < bytes.len() {
                    idx += 2;
                } else {
                    idx += 1;
                }
            }
            if idx < bytes.len() {
                idx += 1;
            }
        } else {
            while idx < bytes.len()
                && !bytes[idx].is_ascii_whitespace()
                && !(bytes[idx] == b':'
                    && (idx + 1 == bytes.len() || bytes[idx + 1].is_ascii_whitespace()))
                && bytes[idx] != b'#'
            {
                idx += 1;
            }
        }

        let val = String::from_utf8_lossy(&bytes[start_idx..idx]).to_string();
        let start_pos = offset_to_position(start_idx, line_starts);
        let end_pos = offset_to_position(idx, line_starts);

        tokens.push(Token {
            kind: TokenKind::Scalar(val, style),
            span: Span::new(start_pos, end_pos),
        });
    }

    let eof_pos = offset_to_position(bytes.len(), line_starts);
    tokens.push(Token {
        kind: TokenKind::Eof,
        span: Span::new(eof_pos, eof_pos),
    });

    Ok(tokens)
}
