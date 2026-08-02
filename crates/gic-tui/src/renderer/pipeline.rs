//! # Render Pipeline

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::Widget;

use crate::renderer::autocomplete_popup::AutocompletePopup;
use crate::renderer::bottom_panel::BottomPanelRenderer;
use crate::renderer::cursor_renderer::{cursor_shape_for_mode, CursorRenderInfo, CursorRenderer};
use crate::renderer::errors::RenderError;
use crate::renderer::file_explorer::FileExplorerRenderer;
use crate::renderer::file_info::FileInfo;
use crate::renderer::intelligence_panel::IntelligencePanelRenderer;
use crate::renderer::layout::{EditorLayout, LayoutEngine, PaneLayout};
use crate::renderer::line_numbers::LineNumberRenderer;
use crate::renderer::quick_fix_popup::QuickFixPopup;
use crate::renderer::render_state::RenderState;
use crate::renderer::status_bar::StatusBarRenderer;
use crate::renderer::syntax::highlighter::PlainTextHighlighter;
use crate::renderer::syntax::languages::LanguageRegistry;
use crate::renderer::syntax::regex_highlighter::RegexHighlighter;
use crate::renderer::syntax::SyntaxHighlighter;
use crate::renderer::text_renderer::TextRenderer;
use crate::renderer::top_bar::TopBarRenderer;
use crate::renderer::types::LineNumberMode;
use gic_core::{Document, TextBuffer};

pub struct RenderPipeline {
    text_renderer: TextRenderer,
    line_number_renderer: LineNumberRenderer,
    cursor_renderer: CursorRenderer,
    language_registry: LanguageRegistry,
    cached_highlighter: Option<Box<dyn SyntaxHighlighter>>,
    cached_highlighter_ext: Option<String>,
}

impl Default for RenderPipeline {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderPipeline {
    pub fn new() -> Self {
        Self {
            text_renderer: TextRenderer::with_default_tab_width(),
            line_number_renderer: LineNumberRenderer::with_default_mode(),
            cursor_renderer: CursorRenderer::with_defaults(),
            language_registry: LanguageRegistry::new(),
            cached_highlighter: None,
            cached_highlighter_ext: None,
        }
    }

    pub fn with_settings(tab_width: usize, line_number_mode: LineNumberMode) -> Self {
        Self {
            text_renderer: TextRenderer::new(tab_width),
            line_number_renderer: LineNumberRenderer::new(line_number_mode),
            cursor_renderer: CursorRenderer::with_defaults(),
            language_registry: LanguageRegistry::new(),
            cached_highlighter: None,
            cached_highlighter_ext: None,
        }
    }

    pub fn line_numbers_mut(&mut self) -> &mut LineNumberRenderer {
        &mut self.line_number_renderer
    }

    pub fn cursor_mut(&mut self) -> &mut CursorRenderer {
        &mut self.cursor_renderer
    }

    pub fn render(
        &mut self,
        buf: &mut Buffer,
        area: Rect,
        state: &RenderState<'_>,
    ) -> Result<CursorRenderInfo, RenderError> {
        let panes = &state.editor.workspace.panes;

        let pane_line_counts: Vec<usize> = panes
            .iter()
            .map(|p| {
                state
                    .editor
                    .workspace
                    .buffers
                    .get(&p.buffer_id)
                    .map(|b| b.line_count())
                    .unwrap_or(1)
            })
            .collect();

        let layout = LayoutEngine::compute(
            area,
            state.file_explorer_open,
            state.intelligence_panel_open,
            state.bottom_panel_open,
            state.command_palette_open,
            &pane_line_counts,
        );

        // 1. Render Top Bar
        self.render_top_bar(buf, &layout, state);

        // 2. Render File Explorer (left panel)
        if let Some(ref explorer_area) = layout.file_explorer_area {
            let explorer = FileExplorerRenderer::new(&state.file_tree, state.theme)
                .with_selection(state.explorer_selected)
                .with_scroll(state.explorer_scroll);
            explorer.render(*explorer_area, buf);
        }

        // 3. Render Intelligence Panel (right panel)
        if let Some(ref intel_area) = layout.intelligence_panel_area {
            let mut panel = IntelligencePanelRenderer::new(&state.diagnostics, state.theme);
            if let Some(ref hover) = state.hover_info {
                panel = panel.with_hover(hover);
            }
            panel.render(*intel_area, buf);
        }

        // 4. Render Bottom Panel
        if let Some(ref bottom_area) = layout.bottom_panel_area {
            let panel = BottomPanelRenderer::new(state.theme);
            panel.render(*bottom_area, buf);
        }

        // 5. Render Editor Panes
        let cursor_shape = cursor_shape_for_mode(&state.editor.engine.active_mode);
        let mut final_cursor_info = CursorRenderInfo {
            screen_position: crate::renderer::types::ScreenPosition::origin(),
            shape: cursor_shape,
            visible: false,
        };

        for (i, pane_layout) in layout.panes.iter().enumerate() {
            let pane = &panes[i];
            let buffer = state.editor.workspace.buffers.get(&pane.buffer_id).unwrap();
            let document = state
                .editor
                .workspace
                .documents
                .get(&pane.buffer_id)
                .unwrap();

            self.render_line_numbers(
                buf,
                pane_layout,
                pane.scroll_row,
                buffer.line_count(),
                pane.cursor.row,
                state.theme,
                &state.diagnostics,
            );
            self.render_text(buf, pane_layout, buffer, document, pane, state);

            // Only capture cursor info for active pane
            if i == state.editor.workspace.active_pane {
                let cursor_line_text = buffer.line(pane.cursor.row);
                let cursor_renderer_copy =
                    CursorRenderer::new(cursor_shape, self.text_renderer.tab_width());

                let mut viewport = crate::renderer::viewport::Viewport::new(
                    pane_layout.text_area.height as usize,
                    pane_layout.text_area.width as usize,
                    buffer.line_count(),
                );
                viewport.set_scroll_row(pane.scroll_row);
                viewport.set_scroll_col(pane.scroll_col);

                final_cursor_info = cursor_renderer_copy.compute(
                    pane.cursor,
                    &viewport,
                    pane_layout.text_area,
                    cursor_line_text,
                );
            }
        }

        // 6. Render Status Bar
        self.render_status_bar(buf, &layout, state);

        // 7. Render Floating Docs Overlays
        if let Some(hover) = &state.hover_info {
            if final_cursor_info.visible {
                crate::renderer::floating_docs::FloatingDocsRenderer::new(
                    hover,
                    state.theme,
                    final_cursor_info.screen_position.col,
                    final_cursor_info.screen_position.row,
                )
                .render(area, buf);
            }
        }

        // 8. Render IntelliSense & Quick Fix Popups
        if final_cursor_info.visible {
            if state.editor.mode == gic_core::EditorMode::Insert && !state.completions.is_empty() {
                AutocompletePopup::new(
                    &state.completions,
                    state.editor.autocomplete_selected_index,
                    state.editor.autocomplete_scroll_offset,
                    state.theme,
                    final_cursor_info.screen_position.col,
                    final_cursor_info.screen_position.row,
                    10,
                )
                .render(area, buf);
            }

            if state.editor.mode == gic_core::EditorMode::Normal && state.editor.quick_fix_menu_open
            {
                // Find diagnostics for the current row that have quick fixes
                let active_pane = &state.editor.workspace.panes[state.editor.workspace.active_pane];
                if let Some(diag) = state
                    .diagnostics
                    .iter()
                    .find(|d| d.row == active_pane.cursor.row && !d.quick_fixes.is_empty())
                {
                    QuickFixPopup::new(
                        &diag.quick_fixes,
                        state.editor.quick_fix_selected_index,
                        state.theme,
                        final_cursor_info.screen_position.col,
                        final_cursor_info.screen_position.row,
                    )
                    .render(area, buf);
                }
            }
        }

        // 9. Render Validation Error Popup
        if let Some(err) = &state.editor.validation_error_popup {
            use ratatui::widgets::{Block, Borders, Paragraph, Clear};
            use ratatui::style::{Style, Color, Modifier};
            use ratatui::text::{Line, Span};
            use ratatui::layout::Alignment;

            let width = 45;
            let height = 15;
            
            let popup_area = Rect {
                x: area.x + (area.width.saturating_sub(width)) / 2,
                y: area.y + (area.height.saturating_sub(height)) / 2,
                width: width.min(area.width),
                height: height.min(area.height),
            };

            Clear.render(popup_area, buf);
            
            let block = Block::default()
                .title(" ❌ YAML Validation Failed ")
                .title_alignment(Alignment::Center)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Red));
                
            let text = vec![
                Line::from(vec![Span::styled(format!("Line: {}", err.row + 1), Style::default().fg(Color::Yellow))]),
                Line::from(vec![Span::styled(format!("Column: {}", err.col + 1), Style::default().fg(Color::Yellow))]),
                Line::from(""),
                Line::from(vec![Span::styled("Problem:", Style::default().add_modifier(Modifier::BOLD))]),
                Line::from(vec![Span::raw(&err.message)]),
                Line::from(""),
                Line::from(vec![Span::styled("Suggestions:", Style::default().add_modifier(Modifier::BOLD))]),
                Line::from("• Check indentation"),
                Line::from("• Replace tabs with spaces"),
                Line::from("• Verify nested blocks"),
                Line::from(""),
                Line::from(vec![Span::styled("Press:", Style::default().add_modifier(Modifier::BOLD))]),
                Line::from("F8     → Jump to Error"),
                Line::from("Ctrl+. → Quick Fix"),
                Line::from("Esc    → Continue Editing"),
            ];

            let p = Paragraph::new(text).block(block).alignment(Alignment::Left);
            p.render(popup_area, buf);
        }

        Ok(final_cursor_info)
    }

    fn render_top_bar(&self, buf: &mut Buffer, layout: &EditorLayout, state: &RenderState<'_>) {
        let active_pane = &state.editor.workspace.panes[state.editor.workspace.active_pane];
        let document = state
            .editor
            .workspace
            .documents
            .get(&active_pane.buffer_id)
            .unwrap();

        let file_name = document
            .path
            .as_ref()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("[Untitled]");
        let file_ext = document
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
            .unwrap_or("Plain Text");

        let top_bar = TopBarRenderer::new(file_name, lang_name, state.theme);
        top_bar.render(layout.top_bar_area, buf);
    }

    fn render_line_numbers(
        &mut self,
        buf: &mut Buffer,
        pane_layout: &PaneLayout,
        scroll_row: usize,
        total_lines: usize,
        cursor_row: usize,
        theme: &crate::renderer::themes::Theme,
        diagnostics: &[gic_core::language_engine::EngineDiagnostic],
    ) {
        if pane_layout.gutter_width == 0 {
            return;
        }

        self.line_number_renderer.render(
            buf,
            pane_layout.line_number_area,
            scroll_row,
            total_lines,
            cursor_row,
            theme,
            diagnostics,
        );
    }

    fn render_text(
        &mut self,
        buf: &mut Buffer,
        pane_layout: &PaneLayout,
        buffer: &TextBuffer,
        document: &Document,
        pane: &gic_core::workspace::pane::EditorPane,
        state: &RenderState<'_>,
    ) {
        if pane_layout.text_area.width == 0 || pane_layout.text_area.height == 0 {
            return;
        }

        let file_ext = document
            .path
            .as_ref()
            .and_then(|p| p.extension())
            .and_then(|e| e.to_str())
            .unwrap_or("");
        let file_name = document
            .path
            .as_ref()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("");

        let current_ext = Some(file_ext.to_string());
        if self.cached_highlighter_ext != current_ext {
            self.cached_highlighter_ext = current_ext;
            let lang_def = self
                .language_registry
                .resolve_by_filename(file_name)
                .or_else(|| self.language_registry.resolve_by_extension(file_ext));

            let highlighter: Box<dyn SyntaxHighlighter> = match lang_def {
                Some(def) => Box::new(RegexHighlighter::new(def)),
                None => Box::new(PlainTextHighlighter),
            };
            self.cached_highlighter = Some(highlighter);
        }

        let full_text = buffer.text();
        if let Some(ref h) = self.cached_highlighter {
            h.update_buffer(&full_text);
        }

        let vis_start = pane.scroll_row;
        let vis_end = vis_start + pane_layout.text_area.height as usize;
        let scroll_col = pane.scroll_col;
        let visible_cols = pane_layout.text_area.width as usize;

        for row_offset in 0..pane_layout.text_area.height as usize {
            let buffer_row = vis_start + row_offset;
            let screen_y = pane_layout.text_area.y + row_offset as u16;

            if buffer_row >= vis_end || buffer_row >= buffer.line_count() {
                let tilde_line = self
                    .text_renderer
                    .render_tilde_line(visible_cols, state.theme);
                let line_area = Rect::new(
                    pane_layout.text_area.x,
                    screen_y,
                    pane_layout.text_area.width,
                    1,
                );
                buf.set_line(line_area.x, line_area.y, &tilde_line, line_area.width);
                continue;
            }

            let line_text = buffer.line(buffer_row).unwrap_or("");
            let is_current_line = buffer_row == pane.cursor.row;

            let tokens = if let Some(ref h) = self.cached_highlighter {
                h.highlight_line(line_text, buffer_row)
            } else {
                vec![]
            };

            let rendered_line = self.text_renderer.render_line(
                line_text,
                &tokens,
                scroll_col,
                visible_cols,
                state.theme,
                is_current_line,
                None, // selection
                buffer_row,
                Some(&state.editor.search_matches), // search results
                Some(state.editor.search_query.as_str()), // search query
                Some(&state.diagnostics),           // diagnostics
                if is_current_line {
                    state.ghost_text.as_deref()
                } else {
                    None
                }, // ghost text
            );

            buf.set_line(
                pane_layout.text_area.x,
                screen_y,
                &rendered_line,
                pane_layout.text_area.width,
            );

            // Inline diagnostics have been removed as they overwrite the subsequent line's text
            // and the user can press F8 or use hover panels to see errors.
        }
    }

    fn render_status_bar(
        &mut self,
        buf: &mut Buffer,
        layout: &EditorLayout,
        state: &RenderState<'_>,
    ) {
        if layout.status_bar_area.height == 0 {
            return;
        }

        let active_pane = &state.editor.workspace.panes[state.editor.workspace.active_pane];
        let document = state
            .editor
            .workspace
            .documents
            .get(&active_pane.buffer_id)
            .unwrap();
        let buffer = state
            .editor
            .workspace
            .buffers
            .get(&active_pane.buffer_id)
            .unwrap();

        let file_name = document
            .path
            .as_ref()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("");
        let file_ext = document
            .path
            .as_ref()
            .and_then(|p| p.extension())
            .and_then(|e| e.to_str())
            .unwrap_or("");

        let lang_name = self
            .language_registry
            .resolve_by_filename(file_name)
            .or_else(|| self.language_registry.resolve_by_extension(file_ext))
            .map(|d| d.name);

        let file_info = FileInfo::from_state(
            document,
            buffer,
            active_pane.cursor.row,
            active_pane.cursor.col,
            lang_name,
        );

        let terminal_size = (
            state.editor.engine.metrics.screen_width,
            state.editor.engine.metrics.screen_height,
        );

        let error_count = state
            .diagnostics
            .iter()
            .filter(|d| d.severity == gic_core::language_engine::EngineSeverity::Error)
            .count();
        let warn_count = state
            .diagnostics
            .iter()
            .filter(|d| d.severity == gic_core::language_engine::EngineSeverity::Warning)
            .count();

        let mut bar = StatusBarRenderer::new(
            &file_info,
            &state.editor.engine.active_mode,
            state.theme,
            terminal_size,
        );

        if !state.editor.engine.status_message.is_empty() {
            bar = bar.with_status_message(&state.editor.engine.status_message);
        }

        if error_count > 0 || warn_count > 0 {
            bar = bar.with_diagnostics(error_count, warn_count);
        }

        bar.render(layout.status_bar_area, buf);
    }
}
