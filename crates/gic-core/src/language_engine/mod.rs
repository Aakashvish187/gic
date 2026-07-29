//! # Language Engine System
//!
//! Modular language engine architecture for GIC.
//! Each supported language/format implements the `LanguageEngine` trait
//! to provide diagnostics, completions, hover documentation, and quick fixes.

pub mod bash_engine;
pub mod context;
pub mod docker_engine;
pub mod generic_engine;
pub mod kubernetes_schema;
pub mod schema;
pub mod spell_checker;
pub mod terraform_engine;
pub mod yaml_engine;

use std::collections::HashMap;

// ─── Core Types ──────────────────────────────────────────────────────

/// Severity level for engine diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EngineSeverity {
    Error,
    Warning,
    Info,
    Hint,
}

impl EngineSeverity {
    pub fn icon(&self) -> &'static str {
        match self {
            Self::Error => "❌",
            Self::Warning => "⚠",
            Self::Info => "ℹ",
            Self::Hint => "💡",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Error => "error",
            Self::Warning => "warning",
            Self::Info => "info",
            Self::Hint => "hint",
        }
    }
}

/// A diagnostic produced by a language engine.
#[derive(Debug, Clone)]
pub struct EngineDiagnostic {
    /// The row (0-indexed) where the diagnostic applies.
    pub row: usize,
    /// The column (0-indexed) where the diagnostic starts.
    pub col: usize,
    /// Length of the affected text span (0 = point diagnostic).
    pub length: usize,
    /// Severity level.
    pub severity: EngineSeverity,
    /// Human-readable diagnostic message.
    pub message: String,
    /// Machine-readable diagnostic code (e.g., "K8S001").
    pub code: Option<String>,
    /// Source engine name (e.g., "kubernetes", "docker").
    pub source: String,
    /// Available quick fixes for this diagnostic.
    pub quick_fixes: Vec<EngineQuickFix>,
}

impl EngineDiagnostic {
    pub fn error(
        row: usize,
        col: usize,
        message: impl Into<String>,
        source: impl Into<String>,
    ) -> Self {
        Self {
            row,
            col,
            length: 0,
            severity: EngineSeverity::Error,
            message: message.into(),
            code: None,
            source: source.into(),
            quick_fixes: Vec::new(),
        }
    }

    pub fn warning(
        row: usize,
        col: usize,
        message: impl Into<String>,
        source: impl Into<String>,
    ) -> Self {
        Self {
            row,
            col,
            length: 0,
            severity: EngineSeverity::Warning,
            message: message.into(),
            code: None,
            source: source.into(),
            quick_fixes: Vec::new(),
        }
    }

    pub fn hint(
        row: usize,
        col: usize,
        message: impl Into<String>,
        source: impl Into<String>,
    ) -> Self {
        Self {
            row,
            col,
            length: 0,
            severity: EngineSeverity::Hint,
            message: message.into(),
            code: None,
            source: source.into(),
            quick_fixes: Vec::new(),
        }
    }

    pub fn with_code(mut self, code: impl Into<String>) -> Self {
        self.code = Some(code.into());
        self
    }

    pub fn with_length(mut self, length: usize) -> Self {
        self.length = length;
        self
    }

    pub fn with_fix(mut self, fix: EngineQuickFix) -> Self {
        self.quick_fixes.push(fix);
        self
    }
}

/// A completion item offered by a language engine.
#[derive(Debug, Clone)]
pub struct Completion {
    /// The text to insert.
    pub insert_text: String,
    /// Display label shown in the completion menu.
    pub label: String,
    /// Short description shown alongside the label.
    pub detail: Option<String>,
    /// The kind of completion (for icon selection).
    pub kind: CompletionKind,
    /// Optional snippet with placeholders (future).
    pub snippet: Option<String>,
}

impl Completion {
    pub fn new(
        label: impl Into<String>,
        insert_text: impl Into<String>,
        kind: CompletionKind,
    ) -> Self {
        Self {
            label: label.into(),
            insert_text: insert_text.into(),
            detail: None,
            kind,
            snippet: None,
        }
    }

    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }
}

/// Categories for completion items.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompletionKind {
    Keyword,
    Value,
    Snippet,
    Property,
    Type,
    Function,
    File,
}

impl CompletionKind {
    pub fn icon(&self) -> &'static str {
        match self {
            Self::Keyword => "⊞",
            Self::Value => "◇",
            Self::Snippet => "⧉",
            Self::Property => "◈",
            Self::Type => "◆",
            Self::Function => "ƒ",
            Self::File => "📄",
        }
    }
}

/// Hover information for a cursor position.
#[derive(Debug, Clone)]
pub struct HoverInfo {
    /// Title/header of the hover tooltip.
    pub title: String,
    /// Description/body text.
    pub description: String,
    /// Syntax example (if available).
    pub syntax: Option<String>,
    /// Best practice notes.
    pub best_practice: Option<String>,
    /// Link to official documentation (for display only).
    pub doc_url: Option<String>,
    pub example: Option<String>,
    pub production_recommendation: Option<String>,
    pub security_recommendation: Option<String>,
    pub common_mistakes: Option<String>,
}

impl HoverInfo {
    pub fn new(title: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            description: description.into(),
            syntax: None,
            best_practice: None,
            doc_url: None,
            example: None,
            production_recommendation: None,
            security_recommendation: None,
            common_mistakes: None,
        }
    }

    pub fn with_syntax(mut self, syntax: impl Into<String>) -> Self {
        self.syntax = Some(syntax.into());
        self
    }

    pub fn with_best_practice(mut self, bp: impl Into<String>) -> Self {
        self.best_practice = Some(bp.into());
        self
    }
}

/// A quick fix action from a language engine.
#[derive(Debug, Clone)]
pub struct EngineQuickFix {
    /// Human-readable title.
    pub title: String,
    /// The row where the fix applies.
    pub row: usize,
    /// The column where the fix starts.
    pub col: usize,
    /// Length of text to replace (0 = insert).
    pub replace_length: usize,
    /// The replacement text.
    pub new_text: String,
    /// Whether this is the preferred/default fix.
    pub is_preferred: bool,
}

impl EngineQuickFix {
    pub fn new(
        title: impl Into<String>,
        row: usize,
        col: usize,
        replace_length: usize,
        new_text: impl Into<String>,
    ) -> Self {
        Self {
            title: title.into(),
            row,
            col,
            replace_length,
            new_text: new_text.into(),
            is_preferred: false,
        }
    }

    pub fn preferred(mut self) -> Self {
        self.is_preferred = true;
        self
    }
}

// ─── Language Engine Trait ────────────────────────────────────────────

/// The core trait that every language engine must implement.
///
/// Language engines are stateless analyzers. They receive the full file
/// content and cursor position, and return diagnostics, completions,
/// hover info, or quick fixes.
pub trait LanguageEngine: Send + Sync {
    /// Returns the engine's display name (e.g., "Kubernetes YAML").
    fn name(&self) -> &'static str;

    /// Returns the engine's identifier (e.g., "kubernetes").
    fn id(&self) -> &'static str;

    /// Analyzes the content and returns diagnostics.
    fn diagnostics(&self, content: &str) -> Vec<EngineDiagnostic>;

    /// Returns completion items for the given cursor position.
    fn completions(&self, content: &str, row: usize, col: usize) -> Vec<Completion>;

    /// Returns hover information for the given cursor position.
    fn hover(&self, content: &str, row: usize, col: usize) -> Option<HoverInfo>;

    /// Returns quick fixes for a specific diagnostic.
    fn quick_fixes(&self, diagnostic: &EngineDiagnostic) -> Vec<EngineQuickFix> {
        diagnostic.quick_fixes.clone()
    }

    /// Formats the given content, returning the formatted string if successful.
    fn format(&self, _content: &str) -> Option<String> {
        None
    }

    /// Smart Enter: Returns a context-aware string to automatically insert when Enter is pressed.
    fn smart_enter(&self, _line: &str) -> Option<String> {
        None
    }

    /// Template Expansion: Returns a full, production-ready block for a given keyword when Tab is pressed.
    fn template_expansion(&self, _keyword: &str) -> Option<String> {
        None
    }
}

// ─── Language Engine Registry ────────────────────────────────────────

/// Maps file types/extensions to their language engines.
pub struct LanguageEngineRegistry {
    engines: HashMap<String, Box<dyn LanguageEngine>>,
    filename_map: HashMap<String, String>,
    extension_map: HashMap<String, String>,
}

impl LanguageEngineRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            engines: HashMap::new(),
            filename_map: HashMap::new(),
            extension_map: HashMap::new(),
        };
        registry.register_defaults();
        registry
    }

    fn register_defaults(&mut self) {
        // YAML / Kubernetes
        self.register_engine(
            Box::new(yaml_engine::YamlEngine::new()),
            &["yaml", "yml"],
            &[],
        );

        // Docker
        self.register_engine(
            Box::new(docker_engine::DockerEngine::new()),
            &[],
            &["Dockerfile", "dockerfile", "Containerfile"],
        );

        // Terraform
        self.register_engine(
            Box::new(terraform_engine::TerraformEngine::new()),
            &["tf", "tfvars"],
            &[],
        );

        // Bash
        self.register_engine(
            Box::new(bash_engine::BashEngine::new()),
            &["sh", "bash", "zsh"],
            &[".bashrc", ".zshrc", ".profile", ".bash_profile"],
        );

        // Generic fallback (registered last, not mapped to any extension)
        self.engines.insert(
            "generic".to_string(),
            Box::new(generic_engine::GenericEngine),
        );
    }

    fn register_engine(
        &mut self,
        engine: Box<dyn LanguageEngine>,
        extensions: &[&str],
        filenames: &[&str],
    ) {
        let id = engine.id().to_string();
        for ext in extensions {
            self.extension_map.insert(ext.to_string(), id.clone());
        }
        for name in filenames {
            self.filename_map.insert(name.to_string(), id.clone());
        }
        self.engines.insert(id, engine);
    }

    /// Resolves the appropriate language engine for a file.
    pub fn resolve(&self, filename: &str, extension: &str) -> &dyn LanguageEngine {
        // Try filename match first
        if let Some(id) = self.filename_map.get(filename) {
            if let Some(engine) = self.engines.get(id) {
                return engine.as_ref();
            }
        }

        // Try extension match
        if let Some(id) = self.extension_map.get(extension) {
            if let Some(engine) = self.engines.get(id) {
                return engine.as_ref();
            }
        }

        // Fallback to generic
        self.engines.get("generic").unwrap().as_ref()
    }

    /// Resolves by analyzing file content (e.g., Kubernetes vs plain YAML).
    pub fn resolve_with_content(
        &self,
        filename: &str,
        extension: &str,
        content: &str,
    ) -> &dyn LanguageEngine {
        // For YAML files, check if it's Kubernetes
        if extension == "yaml" || extension == "yml" {
            if is_kubernetes_yaml(content) {
                return self.engines.get("yaml").unwrap().as_ref();
            }
        }

        self.resolve(filename, extension)
    }
}

impl Default for LanguageEngineRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Heuristic: does this YAML content look like a Kubernetes manifest?
fn is_kubernetes_yaml(content: &str) -> bool {
    let has_api_version = content.lines().any(|l| {
        let trimmed = l.trim();
        trimmed.starts_with("apiVersion:") || trimmed.starts_with("apiversion:")
    });
    let has_kind = content.lines().any(|l| {
        let trimmed = l.trim();
        trimmed.starts_with("kind:")
    });
    has_api_version && has_kind
}
