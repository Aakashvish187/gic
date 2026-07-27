//! Hover Architecture and Interfaces for YAML.
//!
//! Provides trait interfaces and types for future hover documentation, descriptions,
//! examples, and schema references on cursor hover.

use crate::yaml::parser::{Position, Span, YamlAST};

/// Hover tooltip content returned for display.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HoverInfo {
    /// Markdown documentation payload.
    pub markdown: String,
    /// Exact target span of the hovered token or symbol.
    pub span: Span,
    /// Optional schema documentation URL or reference slug.
    pub schema_reference: Option<String>,
    /// Usage examples.
    pub examples: Vec<String>,
}

impl HoverInfo {
    /// Constructs a basic hover tooltip.
    pub fn new(markdown: impl Into<String>, span: Span) -> Self {
        Self {
            markdown: markdown.into(),
            span,
            schema_reference: None,
            examples: Vec::new(),
        }
    }

    /// Attaches a schema documentation reference.
    pub fn with_schema_reference(mut self, reference: impl Into<String>) -> Self {
        self.schema_reference = Some(reference.into());
        self
    }

    /// Attaches code usage examples.
    pub fn with_examples(mut self, examples: Vec<String>) -> Self {
        self.examples = examples;
        self
    }
}

/// Context snapshot under cursor when requesting hover information.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HoverContext {
    /// Cursor position.
    pub position: Position,
    /// Text of line under cursor.
    pub line_text: String,
    /// Path of property keys from document root.
    pub key_path: Vec<String>,
}

/// Trait interface for future hover tooltip providers.
pub trait HoverProvider: Send + Sync {
    /// Provider name.
    fn name(&self) -> &str;

    /// Generates hover tooltip information if available.
    fn hover(&self, ctx: &HoverContext, ast: Option<&YamlAST>) -> Option<HoverInfo>;
}

/// Primary coordinator for YAML hover providers.
#[derive(Default)]
pub struct YamlHoverEngine {
    providers: Vec<Box<dyn HoverProvider>>,
}

impl YamlHoverEngine {
    /// Creates a new YamlHoverEngine.
    pub fn new() -> Self {
        Self {
            providers: Vec::new(),
        }
    }

    /// Registers a hover provider.
    pub fn register_provider(&mut self, provider: Box<dyn HoverProvider>) {
        self.providers.push(provider);
    }

    /// Queries registered hover providers for tooltip content under cursor.
    pub fn hover(&self, ctx: &HoverContext, ast: Option<&YamlAST>) -> Option<HoverInfo> {
        for provider in &self.providers {
            if let Some(info) = provider.hover(ctx, ast) {
                return Some(info);
            }
        }
        None
    }
}
