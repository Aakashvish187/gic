//! # Render State
//!
//! Immutable state bundle passed through the rendering pipeline. This struct
//! aggregates all the data the renderer needs to produce a frame, ensuring
//! the renderer never needs mutable access to any application state.
//!
//! ## Lifetime Design
//!
//! `RenderState<'a>` borrows everything for a single frame. The caller
//! constructs it, passes it through the pipeline, and it is dropped at
//! frame end — no persistent borrowing.

use gic_core::{CursorPosition, Document, EngineState, Selection, TextBuffer};

use crate::renderer::themes::Theme;
use crate::renderer::viewport::Viewport;

/// Immutable snapshot of all application state needed to render a single frame.
///
/// The renderer receives this as `&RenderState` and extracts everything it needs
/// without ever requiring `&mut`. This is the single point of coupling between
/// the application layer and the rendering engine.
///
/// # Future Fields
///
/// Optional fields like `search_results`, `diagnostics`, and `matching_brackets`
/// are included as `Option` types so they can be populated by future milestones
/// without changing the struct's public API.
pub struct RenderState<'a> {
    /// Reference to the text buffer containing all lines.
    pub buffer: &'a TextBuffer,
    /// Reference to the currently open document (file metadata).
    pub document: &'a Document,
    /// Reference to the engine state (mode, status, metrics).
    pub engine_state: &'a EngineState,
    /// Reference to the current viewport (scroll position, visible area).
    pub viewport: &'a Viewport,
    /// Reference to the active theme.
    pub theme: &'a Theme,
    /// Optional active selection.
    pub selection: Option<&'a Selection>,
    /// Current cursor position in buffer coordinates.
    pub cursor_position: CursorPosition,
    /// Optional search result positions for highlighting.
    pub search_results: Option<&'a [CursorPosition]>,
    /// Optional matching bracket position for bracket highlighting.
    pub matching_bracket: Option<CursorPosition>,
    /// Optional git branch name for status bar display.
    pub git_branch: Option<&'a str>,
    /// Optional detected language name for status bar display.
    pub language: Option<&'a str>,
    /// Optional diagnostic error count for status bar.
    pub error_count: Option<usize>,
    /// Optional diagnostic warning count for status bar.
    pub warning_count: Option<usize>,
}

impl<'a> RenderState<'a> {
    /// Creates a new render state with required fields and all optional fields set to `None`.
    pub fn new(
        buffer: &'a TextBuffer,
        document: &'a Document,
        engine_state: &'a EngineState,
        viewport: &'a Viewport,
        theme: &'a Theme,
        cursor_position: CursorPosition,
    ) -> Self {
        Self {
            buffer,
            document,
            engine_state,
            viewport,
            theme,
            selection: None,
            cursor_position,
            search_results: None,
            matching_bracket: None,
            git_branch: None,
            language: None,
            error_count: None,
            warning_count: None,
        }
    }

    /// Sets the active selection for this frame.
    pub fn with_selection(mut self, selection: &'a Selection) -> Self {
        if selection.is_active {
            self.selection = Some(selection);
        }
        self
    }

    /// Sets the language name for the status bar.
    pub fn with_language(mut self, language: &'a str) -> Self {
        self.language = Some(language);
        self
    }

    /// Sets the git branch name for the status bar.
    pub fn with_git_branch(mut self, branch: &'a str) -> Self {
        self.git_branch = Some(branch);
        self
    }

    /// Sets search result highlights.
    pub fn with_search_results(mut self, results: &'a [CursorPosition]) -> Self {
        self.search_results = Some(results);
        self
    }

    /// Sets the matching bracket position.
    pub fn with_matching_bracket(mut self, pos: CursorPosition) -> Self {
        self.matching_bracket = Some(pos);
        self
    }

    /// Sets diagnostic counts.
    pub fn with_diagnostics(mut self, errors: usize, warnings: usize) -> Self {
        self.error_count = Some(errors);
        self.warning_count = Some(warnings);
        self
    }

    /// Returns the total number of lines in the buffer.
    pub fn total_lines(&self) -> usize {
        self.buffer.line_count()
    }

    /// Returns whether the document has been modified since last save.
    pub fn is_modified(&self) -> bool {
        self.buffer.is_modified() || self.document.is_modified
    }

    /// Returns whether the document is read-only.
    pub fn is_read_only(&self) -> bool {
        self.document.is_read_only
    }

    /// Returns the file name for display, or "\[Untitled\]" if no path.
    pub fn display_file_name(&self) -> &str {
        self.document
            .path
            .as_ref()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("[Untitled]")
    }

    /// Returns the current editor mode name.
    pub fn mode_name(&self) -> &str {
        &self.engine_state.active_mode
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::renderer::themes::builtin;
    use gic_core::{Document, EngineState, TextBuffer};

    fn make_test_state<'a>(
        buffer: &'a TextBuffer,
        document: &'a Document,
        engine: &'a EngineState,
        viewport: &'a Viewport,
        theme: &'a Theme,
    ) -> RenderState<'a> {
        RenderState::new(
            buffer,
            document,
            engine,
            viewport,
            theme,
            CursorPosition::zero(),
        )
    }

    #[test]
    fn test_render_state_creation() {
        let buffer = TextBuffer::from_str("Hello World");
        let doc = Document::new_empty();
        let engine = EngineState::new();
        let viewport = Viewport::new(24, 80, 1);
        let theme = builtin::gic_dark();

        let state = make_test_state(&buffer, &doc, &engine, &viewport, &theme);

        assert_eq!(state.total_lines(), 1);
        assert!(!state.is_modified());
        assert!(!state.is_read_only());
        assert_eq!(state.display_file_name(), "[Untitled]");
        assert_eq!(state.mode_name(), "NORMAL");
    }

    #[test]
    fn test_render_state_builder_pattern() {
        let buffer = TextBuffer::from_str("Line 1\nLine 2");
        let doc = Document::new_empty();
        let engine = EngineState::new();
        let viewport = Viewport::new(24, 80, 2);
        let theme = builtin::gic_dark();

        let state = RenderState::new(
            &buffer,
            &doc,
            &engine,
            &viewport,
            &theme,
            CursorPosition::zero(),
        )
        .with_language("Rust")
        .with_git_branch("main")
        .with_diagnostics(2, 5);

        assert_eq!(state.language, Some("Rust"));
        assert_eq!(state.git_branch, Some("main"));
        assert_eq!(state.error_count, Some(2));
        assert_eq!(state.warning_count, Some(5));
    }
}
