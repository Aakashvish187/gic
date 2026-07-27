//! # Render Pipeline
//!
//! The main orchestrator of the rendering engine. The `RenderPipeline` is
//! the single public entry point for rendering a complete frame. It
//! coordinates all sub-renderers (layout, line numbers, text, syntax,
//! cursor, status bar) in the correct order.
//!
//! ## Pipeline Stages
//!
//! 1. **Layout** — Compute region sizes from terminal dimensions.
//! 2. **Line Numbers** — Render gutter with line numbers.
//! 3. **Text + Syntax** — Render visible text with syntax highlighting.
//! 4. **Cursor** — Position terminal cursor.
//! 5. **Status Bar** — Render bottom bar with file info.
//!
//! ## Usage
//!
//! ```text
//! let pipeline = RenderPipeline::new(theme_manager);
//! pipeline.render(frame, &render_state)?;
//! ```

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::Widget;

use crate::renderer::cursor_renderer::{cursor_shape_for_mode, CursorRenderInfo, CursorRenderer};
use crate::renderer::errors::RenderError;
use crate::renderer::file_info::FileInfo;
use crate::renderer::layout::{EditorLayout, LayoutEngine};
use crate::renderer::line_numbers::LineNumberRenderer;
use crate::renderer::render_state::RenderState;
use crate::renderer::status_bar::StatusBarRenderer;
use crate::renderer::syntax::highlighter::PlainTextHighlighter;
use crate::renderer::syntax::languages::LanguageRegistry;
use crate::renderer::syntax::regex_highlighter::RegexHighlighter;
use crate::renderer::syntax::SyntaxHighlighter;
use crate::renderer::text_renderer::TextRenderer;
use crate::renderer::types::LineNumberMode;

/// The main rendering pipeline orchestrator.
///
/// The pipeline is constructed once and reused across frames. It owns
/// the sub-renderers and coordinates them to produce complete frames.
pub struct RenderPipeline {
    /// Text renderer for line content.
    text_renderer: TextRenderer,
    /// Line number gutter renderer.
    line_number_renderer: LineNumberRenderer,
    /// Cursor position calculator.
    cursor_renderer: CursorRenderer,
    /// Language registry for syntax highlighting resolution.
    language_registry: LanguageRegistry,
}

impl RenderPipeline {
    /// Creates a new render pipeline with default settings.
    pub fn new() -> Self {
        Self {
            text_renderer: TextRenderer::with_default_tab_width(),
            line_number_renderer: LineNumberRenderer::with_default_mode(),
            cursor_renderer: CursorRenderer::with_defaults(),
            language_registry: LanguageRegistry::new(),
        }
    }

    /// Creates a render pipeline with custom tab width and line number mode.
    pub fn with_settings(tab_width: usize, line_number_mode: LineNumberMode) -> Self {
        Self {
            text_renderer: TextRenderer::new(tab_width),
            line_number_renderer: LineNumberRenderer::new(line_number_mode),
            cursor_renderer: CursorRenderer::with_defaults(),
            language_registry: LanguageRegistry::new(),
        }
    }

    /// Returns a mutable reference to the line number renderer for configuration.
    pub fn line_numbers_mut(&mut self) -> &mut LineNumberRenderer {
        &mut self.line_number_renderer
    }

    /// Returns a mutable reference to the cursor renderer for configuration.
    pub fn cursor_mut(&mut self) -> &mut CursorRenderer {
        &mut self.cursor_renderer
    }

    /// Renders a complete frame into the given ratatui buffer.
    ///
    /// This is the primary public API of the rendering engine. It takes
    /// an immutable `RenderState` and produces a complete frame.
    ///
    /// # Arguments
    ///
    /// * `buf` - Ratatui buffer to render into.
    /// * `area` - Full terminal area.
    /// * `state` - Immutable render state for this frame.
    ///
    /// # Returns
    ///
    /// A `CursorRenderInfo` indicating where to place the terminal cursor,
    /// or a `RenderError` if rendering fails.
    pub fn render(
        &self,
        buf: &mut Buffer,
        area: Rect,
        state: &RenderState<'_>,
    ) -> Result<CursorRenderInfo, RenderError> {
        // Stage 1: Layout
        let layout = LayoutEngine::compute(area, state.total_lines());

        // Update cursor shape based on mode
        let cursor_shape = cursor_shape_for_mode(state.mode_name());

        // Stage 2: Line Numbers
        self.render_line_numbers(buf, &layout, state);

        // Stage 3: Text + Syntax
        self.render_text(buf, &layout, state);

        // Stage 4: Status Bar
        self.render_status_bar(buf, &layout, state);

        // Stage 5: Compute cursor position
        let cursor_line_text = state.buffer.line(state.cursor_position.row);
        let cursor_renderer_copy =
            CursorRenderer::new(cursor_shape, self.text_renderer.tab_width());
        let cursor_info = cursor_renderer_copy.compute(
            state.cursor_position,
            state.viewport,
            layout.text_area,
            cursor_line_text,
        );

        Ok(cursor_info)
    }

    /// Renders line numbers into the gutter area.
    fn render_line_numbers(
        &self,
        buf: &mut Buffer,
        layout: &EditorLayout,
        state: &RenderState<'_>,
    ) {
        if layout.gutter_width == 0 {
            return;
        }

        self.line_number_renderer.render(
            buf,
            layout.line_number_area,
            state.viewport.scroll_row(),
            state.total_lines(),
            state.cursor_position.row,
            state.theme,
        );
    }

    /// Renders text content with syntax highlighting.
    fn render_text(&self, buf: &mut Buffer, layout: &EditorLayout, state: &RenderState<'_>) {
        if !layout.has_text_area() {
            return;
        }

        // Resolve syntax highlighter
        let file_ext = state
            .document
            .path
            .as_ref()
            .and_then(|p| p.extension())
            .and_then(|e| e.to_str())
            .unwrap_or("");
        let file_name = state
            .document
            .path
            .as_ref()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("");

        let lang_def = self
            .language_registry
            .resolve_by_filename(file_name)
            .or_else(|| self.language_registry.resolve_by_extension(file_ext));

        // Create highlighter (or plain text fallback)
        let regex_highlighter;
        let plain_highlighter = PlainTextHighlighter;
        let highlighter: &dyn SyntaxHighlighter = match lang_def {
            Some(def) => {
                regex_highlighter = RegexHighlighter::new(def);
                &regex_highlighter
            }
            None => &plain_highlighter,
        };

        let (vis_start, vis_end) = state.viewport.visible_line_range();
        let scroll_col = state.viewport.scroll_col();
        let visible_cols = layout.text_cols() as usize;

        for row_offset in 0..layout.text_rows() as usize {
            let buffer_row = vis_start + row_offset;
            let screen_y = layout.text_area.y + row_offset as u16;

            if buffer_row >= vis_end || buffer_row >= state.total_lines() {
                // Past end of buffer — render tilde line
                let tilde_line = self
                    .text_renderer
                    .render_tilde_line(visible_cols, state.theme);
                let line_area = Rect::new(layout.text_area.x, screen_y, layout.text_area.width, 1);
                buf.set_line(line_area.x, line_area.y, &tilde_line, line_area.width);
                continue;
            }

            let line_text = state.buffer.line(buffer_row).unwrap_or("");
            let is_current_line = buffer_row == state.cursor_position.row;

            // Highlight line
            let tokens = highlighter.highlight_line(line_text, buffer_row);

            // Render line
            let rendered_line = self.text_renderer.render_line(
                line_text,
                &tokens,
                scroll_col,
                visible_cols,
                state.theme,
                is_current_line,
                state.selection,
                buffer_row,
            );

            buf.set_line(
                layout.text_area.x,
                screen_y,
                &rendered_line,
                layout.text_area.width,
            );
        }
    }

    /// Renders the status bar.
    fn render_status_bar(&self, buf: &mut Buffer, layout: &EditorLayout, state: &RenderState<'_>) {
        if layout.status_bar_area.height == 0 {
            return;
        }

        // Resolve language name
        let file_name = state
            .document
            .path
            .as_ref()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("");
        let file_ext = state
            .document
            .path
            .as_ref()
            .and_then(|p| p.extension())
            .and_then(|e| e.to_str())
            .unwrap_or("");

        let lang_name = self
            .language_registry
            .resolve_by_filename(file_name)
            .or_else(|| self.language_registry.resolve_by_extension(file_ext))
            .map(|d| d.name)
            .or(state.language);

        let file_info = FileInfo::from_state(
            state.document,
            state.buffer,
            state.cursor_position.row,
            state.cursor_position.col,
            lang_name,
        );

        let terminal_size = (
            state.engine_state.metrics.screen_width,
            state.engine_state.metrics.screen_height,
        );

        let mut bar =
            StatusBarRenderer::new(&file_info, state.mode_name(), state.theme, terminal_size);

        if !state.engine_state.status_message.is_empty() {
            bar = bar.with_status_message(&state.engine_state.status_message);
        }

        if let Some(branch) = state.git_branch {
            bar = bar.with_git_branch(branch);
        }

        if let (Some(errors), Some(warnings)) = (state.error_count, state.warning_count) {
            bar = bar.with_diagnostics(errors, warnings);
        }

        bar.render(layout.status_bar_area, buf);
    }
}

impl Default for RenderPipeline {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::renderer::themes::builtin;
    use crate::renderer::viewport::Viewport;
    use gic_core::{CursorPosition, Document, EngineState, TextBuffer};

    #[test]
    fn test_pipeline_creation() {
        let pipeline = RenderPipeline::new();
        assert!(std::mem::size_of_val(&pipeline) > 0);
    }

    #[test]
    fn test_pipeline_render_empty_buffer() {
        let pipeline = RenderPipeline::new();
        let buffer = TextBuffer::new();
        let doc = Document::new_empty();
        let engine = EngineState::new();
        let viewport = Viewport::new(23, 80, buffer.line_count());
        let theme = builtin::gic_dark();

        let state = RenderState::new(
            &buffer,
            &doc,
            &engine,
            &viewport,
            &theme,
            CursorPosition::zero(),
        );

        let area = Rect::new(0, 0, 80, 24);
        let mut buf = Buffer::empty(area);

        let result = pipeline.render(&mut buf, area, &state);
        assert!(result.is_ok());

        let cursor_info = result.unwrap();
        assert!(cursor_info.visible);
    }

    #[test]
    fn test_pipeline_render_with_content() {
        let pipeline = RenderPipeline::new();
        let buffer = TextBuffer::from_str("fn main() {\n    println!(\"Hello, GIC!\");\n}");
        let mut doc = Document::new_empty();
        doc.set_path("main.rs");
        let engine = EngineState::new();
        let viewport = Viewport::new(23, 80, buffer.line_count());
        let theme = builtin::gic_dark();

        let state = RenderState::new(
            &buffer,
            &doc,
            &engine,
            &viewport,
            &theme,
            CursorPosition::new(1, 4),
        );

        let area = Rect::new(0, 0, 80, 24);
        let mut buf = Buffer::empty(area);

        let result = pipeline.render(&mut buf, area, &state);
        assert!(result.is_ok());

        // Check that content was rendered
        let content: String = buf
            .content()
            .iter()
            .map(|c| c.symbol().to_string())
            .collect();
        assert!(content.contains("main"));
        assert!(content.contains("println"));
    }

    #[test]
    fn test_pipeline_render_large_file() {
        let pipeline = RenderPipeline::new();
        let lines: Vec<String> = (0..10_000)
            .map(|i| format!("line_{}: let x = {};", i, i))
            .collect();
        let buffer = TextBuffer::from_lines(lines);
        let doc = Document::new_empty();
        let engine = EngineState::new();
        let viewport = Viewport::new(23, 80, buffer.line_count());
        let theme = builtin::gic_dark();

        let state = RenderState::new(
            &buffer,
            &doc,
            &engine,
            &viewport,
            &theme,
            CursorPosition::zero(),
        );

        let area = Rect::new(0, 0, 80, 24);
        let mut buf = Buffer::empty(area);

        let result = pipeline.render(&mut buf, area, &state);
        assert!(result.is_ok());
    }

    #[test]
    fn test_pipeline_tiny_terminal() {
        let pipeline = RenderPipeline::new();
        let buffer = TextBuffer::from_str("Hello");
        let doc = Document::new_empty();
        let engine = EngineState::new();
        let viewport = Viewport::new(1, 5, buffer.line_count());
        let theme = builtin::gic_dark();

        let state = RenderState::new(
            &buffer,
            &doc,
            &engine,
            &viewport,
            &theme,
            CursorPosition::zero(),
        );

        let area = Rect::new(0, 0, 5, 2);
        let mut buf = Buffer::empty(area);

        let result = pipeline.render(&mut buf, area, &state);
        assert!(result.is_ok());
    }

    #[test]
    fn test_pipeline_with_settings() {
        let pipeline = RenderPipeline::with_settings(2, LineNumberMode::Relative);
        assert!(std::mem::size_of_val(&pipeline) > 0);
    }
}
