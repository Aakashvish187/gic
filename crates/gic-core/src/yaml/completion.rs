//! Autocomplete Architecture and Interfaces for YAML.
//!
//! Provides trait interfaces and types for future key, value, snippet, schema-aware,
//! and context-aware completion backends.

use crate::yaml::parser::{Position, YamlAST};

/// Classification of suggested completion item.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CompletionKind {
    #[default]
    Key,
    Value,
    Snippet,
    Property,
    EnumVariant,
    Anchor,
    Alias,
}

/// Single completion recommendation displayed in UI popup.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompletionItem {
    /// Text label shown in completion list.
    pub label: String,
    /// Code or text inserted upon selection.
    pub insert_text: String,
    /// Brief inline details or type information.
    pub detail: Option<String>,
    /// Extended markdown documentation string.
    pub documentation: Option<String>,
    /// Kind of completion proposal.
    pub kind: CompletionKind,
    /// Sorting priority string.
    pub sort_text: Option<String>,
}

impl CompletionItem {
    /// Creates a new completion item with required fields.
    pub fn new(
        label: impl Into<String>,
        insert_text: impl Into<String>,
        kind: CompletionKind,
    ) -> Self {
        Self {
            label: label.into(),
            insert_text: insert_text.into(),
            detail: None,
            documentation: None,
            kind,
            sort_text: None,
        }
    }

    /// Attaches detail text.
    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    /// Attaches documentation.
    pub fn with_documentation(mut self, doc: impl Into<String>) -> Self {
        self.documentation = Some(doc.into());
        self
    }
}

/// Context snapshot at the editor cursor location when requesting autocomplete.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompletionContext {
    /// Current position of the editor cursor.
    pub position: Position,
    /// Line string under cursor.
    pub line_text: String,
    /// Trigger character (e.g. `:` or `-` or `*` or `&`).
    pub trigger_character: Option<char>,
    /// Key path hierarchy leading to current node (e.g. `["metadata", "labels"]`).
    pub key_path: Vec<String>,
}

/// Trait interface for future autocomplete providers.
pub trait CompletionProvider: Send + Sync {
    /// Provider identifier.
    fn name(&self) -> &str;

    /// Generates completion candidates given the editor context and AST.
    fn complete(&self, ctx: &CompletionContext, ast: Option<&YamlAST>) -> Vec<CompletionItem>;
}

/// Primary coordinator for YAML completion providers.
#[derive(Default)]
pub struct YamlCompletionEngine {
    providers: Vec<Box<dyn CompletionProvider>>,
}

impl YamlCompletionEngine {
    /// Creates a new empty YamlCompletionEngine.
    pub fn new() -> Self {
        Self {
            providers: Vec::new(),
        }
    }

    /// Registers a completion provider.
    pub fn register_provider(&mut self, provider: Box<dyn CompletionProvider>) {
        self.providers.push(provider);
    }

    /// Queries all registered completion providers and aggregates results.
    pub fn complete(&self, ctx: &CompletionContext, ast: Option<&YamlAST>) -> Vec<CompletionItem> {
        let mut results = Vec::new();
        for provider in &self.providers {
            results.extend(provider.complete(ctx, ast));
        }
        results
    }
}
