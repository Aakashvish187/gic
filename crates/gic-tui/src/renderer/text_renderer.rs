//! # Text Renderer
//!
//! Renders visible text lines into sequences of styled ratatui `Span`s.
//! Handles tab expansion, Unicode width calculations, horizontal scroll
//! clipping, and composition of syntax highlighting with selection overlays.
//!
//! ## Performance
//!
//! Each visible line is processed in a single pass. The renderer iterates
//! characters once, computing display widths and applying styles simultaneously.
//! No intermediate string allocations are made for clipping — we use character
//! iterators with skip/take semantics.

use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use unicode_width::UnicodeWidthChar;

use gic_core::{CursorPosition, Selection};

use crate::renderer::syntax::HighlightedToken;
use crate::renderer::themes::Theme;

/// Default tab display width in columns.
const DEFAULT_TAB_WIDTH: usize = 4;

/// Renders text content into styled ratatui spans.
///
/// The text renderer is stateless — it takes line content, styling information,
/// and viewport parameters, and produces `Line` values ready for ratatui rendering.
pub struct TextRenderer {
    /// Tab display width in columns.
    tab_width: usize,
}

impl TextRenderer {
    /// Creates a new text renderer with the given tab width.
    pub fn new(tab_width: usize) -> Self {
        Self {
            tab_width: if tab_width == 0 {
                DEFAULT_TAB_WIDTH
            } else {
                tab_width
            },
        }
    }

    /// Creates a text renderer with the default tab width (4).
    pub fn with_default_tab_width() -> Self {
        Self::new(DEFAULT_TAB_WIDTH)
    }

    /// Returns the configured tab width.
    pub fn tab_width(&self) -> usize {
        self.tab_width
    }

    /// Renders a single line of text into a styled ratatui `Line`.
    ///
    /// # Arguments
    ///
    /// * `line_text` - The raw text content of the line.
    /// * `tokens` - Syntax-highlighted tokens for this line.
    /// * `scroll_col` - Horizontal scroll offset (columns to skip from left).
    /// * `visible_cols` - Maximum number of display columns to render.
    /// * `theme` - Active theme for styling.
    /// * `is_current_line` - Whether this is the line under the cursor.
    /// * `selection` - Optional active selection for overlay.
    /// * `line_row` - Buffer row index of this line (for selection calculation).
    ///
    /// # Returns
    ///
    /// A ratatui `Line` containing styled `Span`s clipped to the visible viewport.
    #[allow(clippy::too_many_arguments)]
    pub fn render_line(
        &self,
        line_text: &str,
        tokens: &[HighlightedToken],
        scroll_col: usize,
        visible_cols: usize,
        theme: &Theme,
        is_current_line: bool,
        selection: Option<&Selection>,
        line_row: usize,
        search_results: Option<&[CursorPosition]>,
        search_query: Option<&str>,
        diagnostics: Option<&[gic_core::language_engine::EngineDiagnostic]>,
        ghost_text: Option<&str>,
    ) -> Line<'static> {
        if visible_cols == 0 {
            return Line::default();
        }

        let base_bg = if is_current_line {
            theme.cursor_line
        } else {
            theme.background
        };

        // If no tokens, render as plain text
        if tokens.is_empty() {
            return self.render_plain_line(
                line_text,
                scroll_col,
                visible_cols,
                base_bg,
                theme.foreground,
                selection,
                line_row,
                search_results,
                search_query,
                theme,
                diagnostics,
            );
        }

        // Build spans from tokens with horizontal clipping
        let mut spans: Vec<Span<'static>> = Vec::new();
        let mut display_col: usize = 0;

        for token in tokens {
            let token_style = theme.style_for_token(token.kind);
            let fg = token_style.fg.unwrap_or(theme.foreground);

            for ch in token.text.chars() {
                let ch_width = self.char_display_width(ch);

                // Skip characters before scroll offset
                if display_col + ch_width <= scroll_col {
                    display_col += ch_width;
                    continue;
                }

                // Stop if we've filled the visible area
                let viewport_col = display_col.saturating_sub(scroll_col);
                if viewport_col >= visible_cols {
                    break;
                }

                // Determine if this character is selected
                let is_selected = self.is_char_selected(selection, line_row, display_col);
                
                // Determine if this character is a search match
                let is_search_match = search_results.map_or(false, |results| {
                    let query_len = search_query.map_or(1, |q| q.chars().count().max(1));
                    results.iter().any(|pos| pos.row == line_row && display_col >= pos.col && display_col < pos.col + query_len)
                });

                let mut char_bg = if is_selected {
                    theme.selection
                } else {
                    base_bg
                };
                
                let mut char_fg = fg;
                let mut modifier = token_style.add_modifier;
                
                if is_search_match && !is_selected {
                    char_bg = Color::Yellow;
                    char_fg = Color::Black;
                }

                // Check for diagnostics overlapping this character
                if let Some(diags) = diagnostics {
                    for d in diags {
                        if d.row == line_row && display_col >= d.col && display_col < d.col + d.length {
                            modifier = modifier | Modifier::UNDERLINED;
                            if d.severity == gic_core::language_engine::EngineSeverity::Error {
                                char_fg = theme.diagnostic_error;
                            } else if d.severity == gic_core::language_engine::EngineSeverity::Warning && char_fg != theme.diagnostic_error {
                                char_fg = theme.diagnostic_warning;
                            }
                        }
                    }
                }

                let style = Style::default()
                    .fg(char_fg)
                    .bg(char_bg)
                    .add_modifier(modifier);

                let display_text = if ch == '\t' {
                    " ".repeat(ch_width)
                } else {
                    ch.to_string()
                };

                spans.push(Span::styled(display_text, style));
                display_col += ch_width;
            }
        }

        if let Some(gt) = ghost_text {
            let gt_style = Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC);
            let display_gt = if let Some(idx) = gt.find('\n') {
                format!("{}...", &gt[..idx])
            } else {
                gt.to_string()
            };
            spans.push(Span::styled(display_gt, gt_style));
            // display_col isn't increased because ghost text isn't part of the real buffer width
        }

        // Fill remaining viewport with background
        let rendered_cols = display_col.saturating_sub(scroll_col).min(visible_cols);
        if rendered_cols < visible_cols {
            let fill_width = visible_cols - rendered_cols;

            // Check if the fill area falls within a selection
            let fill_bg = if self.is_range_selected(
                selection,
                line_row,
                display_col,
                display_col + fill_width,
            ) {
                theme.selection
            } else {
                base_bg
            };

            spans.push(Span::styled(
                " ".repeat(fill_width),
                Style::default().bg(fill_bg),
            ));
        }

        Line::from(spans)
    }

    /// Renders a plain (un-highlighted) line.
    fn render_plain_line(
        &self,
        line_text: &str,
        scroll_col: usize,
        visible_cols: usize,
        base_bg: Color,
        base_fg: Color,
        selection: Option<&Selection>,
        line_row: usize,
        search_results: Option<&[CursorPosition]>,
        search_query: Option<&str>,
        theme: &Theme,
        diagnostics: Option<&[gic_core::language_engine::EngineDiagnostic]>,
    ) -> Line<'static> {
        let mut spans: Vec<Span<'static>> = Vec::new();
        let mut display_col: usize = 0;

        for ch in line_text.chars() {
            let ch_width = self.char_display_width(ch);

            if display_col + ch_width <= scroll_col {
                display_col += ch_width;
                continue;
            }

            let viewport_col = display_col.saturating_sub(scroll_col);
            if viewport_col >= visible_cols {
                break;
            }

            // Determine if this character is selected
            let is_selected = self.is_char_selected(selection, line_row, display_col);
            
            // Determine if this character is a search match
            let is_search_match = search_results.map_or(false, |results| {
                let query_len = search_query.map_or(1, |q| q.chars().count().max(1));
                results.iter().any(|pos| pos.row == line_row && display_col >= pos.col && display_col < pos.col + query_len)
            });

            let mut char_bg = if is_selected {
                theme.selection
            } else {
                base_bg
            };
            
            let mut char_fg = base_fg;
            let mut modifier = Modifier::empty();
            
            if is_search_match && !is_selected {
                char_bg = Color::Yellow;
                char_fg = Color::Black;
            }

            // Check for diagnostics overlapping this character
            if let Some(diags) = diagnostics {
                for d in diags {
                    if d.row == line_row && display_col >= d.col && display_col < d.col + d.length {
                        modifier = modifier | Modifier::UNDERLINED;
                        if d.severity == gic_core::language_engine::EngineSeverity::Error {
                            char_fg = theme.diagnostic_error;
                        } else if d.severity == gic_core::language_engine::EngineSeverity::Warning && char_fg != theme.diagnostic_error {
                            char_fg = theme.diagnostic_warning;
                        }
                    }
                }
            }

            let style = Style::default().fg(char_fg).bg(char_bg).add_modifier(modifier);

            let display_text = if ch == '\t' {
                " ".repeat(ch_width)
            } else {
                ch.to_string()
            };

            spans.push(Span::styled(display_text, style));
            display_col += ch_width;
        }

        // Fill remaining
        let rendered_cols = display_col.saturating_sub(scroll_col).min(visible_cols);
        if rendered_cols < visible_cols {
            spans.push(Span::styled(
                " ".repeat(visible_cols - rendered_cols),
                Style::default().bg(base_bg),
            ));
        }

        Line::from(spans)
    }

    /// Computes the display width of a character.
    ///
    /// Tabs expand to `tab_width` columns. Control characters are width 0.
    /// CJK characters are width 2. All others use Unicode width.
    pub fn char_display_width(&self, ch: char) -> usize {
        match ch {
            '\t' => self.tab_width,
            c if c.is_control() => 0,
            c => UnicodeWidthChar::width(c).unwrap_or(0),
        }
    }

    /// Computes the total display width of a string.
    pub fn string_display_width(&self, s: &str) -> usize {
        s.chars().map(|c| self.char_display_width(c)).sum()
    }

    /// Computes the display column offset for a given character index in a line.
    ///
    /// This converts a buffer column (character index) to a display column
    /// (accounting for tab expansion and wide characters).
    pub fn char_index_to_display_col(&self, line: &str, char_index: usize) -> usize {
        line.chars()
            .take(char_index)
            .map(|c| self.char_display_width(c))
            .sum()
    }

    /// Checks if a character at the given buffer position is within an active selection.
    fn is_char_selected(
        &self,
        selection: Option<&Selection>,
        row: usize,
        _display_col: usize,
    ) -> bool {
        match selection {
            Some(sel) if sel.is_active => {
                let pos = CursorPosition::new(row, _display_col);
                sel.contains(pos)
            }
            _ => false,
        }
    }

    /// Checks if any part of a display column range is selected.
    fn is_range_selected(
        &self,
        selection: Option<&Selection>,
        row: usize,
        start_col: usize,
        _end_col: usize,
    ) -> bool {
        self.is_char_selected(selection, row, start_col)
    }

    /// Renders an empty line (just background fill).
    pub fn render_empty_line(&self, visible_cols: usize, bg: Color) -> Line<'static> {
        Line::from(vec![Span::styled(
            " ".repeat(visible_cols),
            Style::default().bg(bg),
        )])
    }

    /// Renders the `~` placeholder for lines beyond the end of the buffer
    /// (similar to vim's tilde lines).
    pub fn render_tilde_line(&self, visible_cols: usize, theme: &Theme) -> Line<'static> {
        let mut spans = Vec::with_capacity(2);
        spans.push(Span::styled(
            "~",
            Style::default()
                .fg(theme.line_number)
                .bg(theme.background)
                .add_modifier(Modifier::DIM),
        ));
        if visible_cols > 1 {
            spans.push(Span::styled(
                " ".repeat(visible_cols - 1),
                Style::default().bg(theme.background),
            ));
        }
        Line::from(spans)
    }
}


