//! # Line Number Renderer
//!
//! Renders line numbers in the gutter area with support for absolute,
//! relative, and hybrid numbering modes. Highlights the current line
//! number and synchronizes with the viewport scroll offset.
//!
//! ## Performance
//!
//! Rendering is O(visible_lines) — only numbers for visible lines are
//! computed and formatted.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;

use crate::renderer::themes::Theme;
use crate::renderer::types::LineNumberMode;

/// Renders line numbers in the editor gutter.
///
/// The line number renderer is stateless — it takes viewport state and
/// theme, and renders directly into a ratatui `Buffer`.
pub struct LineNumberRenderer {
    /// Line numbering mode.
    mode: LineNumberMode,
}

impl LineNumberRenderer {
    /// Creates a new line number renderer with the specified mode.
    pub fn new(mode: LineNumberMode) -> Self {
        Self { mode }
    }

    /// Creates a line number renderer with the default mode (Absolute).
    pub fn with_default_mode() -> Self {
        Self::new(LineNumberMode::Absolute)
    }

    /// Returns the current line number mode.
    pub fn mode(&self) -> LineNumberMode {
        self.mode
    }

    /// Sets the line number display mode.
    pub fn set_mode(&mut self, mode: LineNumberMode) {
        self.mode = mode;
    }

    /// Renders line numbers for the visible viewport range.
    ///
    /// # Arguments
    ///
    /// * `buf` - Ratatui buffer to render into.
    /// * `area` - The gutter `Rect` area.
    /// * `scroll_row` - First visible buffer row.
    /// * `total_lines` - Total number of lines in the buffer.
    /// * `current_line` - The buffer row where the cursor is.
    /// * `theme` - Active theme for styling.
    pub fn render(
        &self,
        buf: &mut Buffer,
        area: Rect,
        scroll_row: usize,
        total_lines: usize,
        current_line: usize,
        theme: &Theme,
    ) {
        if area.width == 0 || area.height == 0 {
            return;
        }

        // Fill gutter background
        let gutter_style = Style::default().bg(theme.gutter_bg);
        for y in area.y..area.y + area.height {
            for x in area.x..area.x + area.width {
                buf.get_mut(x, y).set_style(gutter_style);
                buf.get_mut(x, y).set_symbol(" ");
            }
        }

        let display_width = (area.width as usize).saturating_sub(1); // 1 col right padding

        for row_offset in 0..area.height as usize {
            let buffer_row = scroll_row + row_offset;
            let screen_y = area.y + row_offset as u16;

            if buffer_row >= total_lines {
                // Past end of buffer — render empty or tilde
                break;
            }

            let is_current = buffer_row == current_line;

            let number_text = self.format_line_number(buffer_row, current_line, display_width);
            let style = if is_current {
                theme.active_line_number_style()
            } else {
                theme.line_number_style()
            };

            // Right-align the number within the gutter (leaving 1 col padding on right)
            let text_len = number_text.len();
            let x_offset = if text_len < display_width {
                (display_width - text_len) as u16
            } else {
                0
            };

            for (i, ch) in number_text.chars().enumerate() {
                let x = area.x + x_offset + i as u16;
                if x < area.x + area.width - 1 {
                    buf.get_mut(x, screen_y).set_symbol(&ch.to_string());
                    buf.get_mut(x, screen_y).set_style(style);
                }
            }
        }
    }

    /// Formats a line number according to the current mode.
    ///
    /// # Arguments
    ///
    /// * `buffer_row` - The 0-indexed buffer row.
    /// * `current_line` - The 0-indexed cursor row.
    /// * `max_width` - Maximum character width for the formatted number.
    fn format_line_number(
        &self,
        buffer_row: usize,
        current_line: usize,
        max_width: usize,
    ) -> String {
        let display_number = match self.mode {
            LineNumberMode::Absolute => buffer_row + 1, // 1-indexed for display
            LineNumberMode::Relative => {
                if buffer_row == current_line {
                    buffer_row + 1 // Show absolute on cursor line
                } else {
                    
                    buffer_row.abs_diff(current_line)
                }
            }
            LineNumberMode::Hybrid => {
                if buffer_row == current_line {
                    buffer_row + 1 // Show absolute on cursor line
                } else {
                    
                    buffer_row.abs_diff(current_line)
                }
            }
        };

        let formatted = display_number.to_string();
        if formatted.len() > max_width {
            formatted[formatted.len() - max_width..].to_string()
        } else {
            formatted
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::renderer::themes::builtin;

    #[test]
    fn test_line_number_absolute() {
        let renderer = LineNumberRenderer::new(LineNumberMode::Absolute);
        assert_eq!(renderer.format_line_number(0, 5, 5), "1");
        assert_eq!(renderer.format_line_number(9, 5, 5), "10");
        assert_eq!(renderer.format_line_number(99, 5, 5), "100");
    }

    #[test]
    fn test_line_number_relative() {
        let renderer = LineNumberRenderer::new(LineNumberMode::Relative);

        // Current line shows absolute
        assert_eq!(renderer.format_line_number(5, 5, 5), "6");

        // Lines above show distance
        assert_eq!(renderer.format_line_number(3, 5, 5), "2");

        // Lines below show distance
        assert_eq!(renderer.format_line_number(8, 5, 5), "3");
    }

    #[test]
    fn test_line_number_hybrid() {
        let renderer = LineNumberRenderer::new(LineNumberMode::Hybrid);

        // Current line shows absolute
        assert_eq!(renderer.format_line_number(10, 10, 5), "11");

        // Other lines show relative distance
        assert_eq!(renderer.format_line_number(7, 10, 5), "3");
        assert_eq!(renderer.format_line_number(12, 10, 5), "2");
    }

    #[test]
    fn test_line_number_truncation() {
        let renderer = LineNumberRenderer::new(LineNumberMode::Absolute);
        // If max_width is smaller than number, show rightmost digits
        assert_eq!(renderer.format_line_number(9999, 0, 2), "00");
    }

    #[test]
    fn test_render_to_buffer() {
        let renderer = LineNumberRenderer::with_default_mode();
        let theme = builtin::gic_dark();
        let area = Rect::new(0, 0, 5, 3);
        let mut buf = Buffer::empty(area);

        renderer.render(&mut buf, area, 0, 10, 1, &theme);

        // Verify that the buffer was written to (not empty)
        let content: String = buf
            .content()
            .iter()
            .map(|c| c.symbol().to_string())
            .collect();
        assert!(content.contains('1'));
        assert!(content.contains('2'));
        assert!(content.contains('3'));
    }

    #[test]
    fn test_render_past_buffer_end() {
        let renderer = LineNumberRenderer::with_default_mode();
        let theme = builtin::gic_dark();
        let area = Rect::new(0, 0, 5, 10); // 10 rows but only 3 lines
        let mut buf = Buffer::empty(area);

        renderer.render(&mut buf, area, 0, 3, 0, &theme);
        // Should not crash, lines 4-10 should be empty
    }

    #[test]
    fn test_render_with_scroll() {
        let renderer = LineNumberRenderer::new(LineNumberMode::Absolute);
        let theme = builtin::gic_dark();
        let area = Rect::new(0, 0, 5, 3);
        let mut buf = Buffer::empty(area);

        renderer.render(&mut buf, area, 50, 100, 51, &theme);

        let content: String = buf
            .content()
            .iter()
            .map(|c| c.symbol().to_string())
            .collect();
        // Should show lines 51, 52, 53 (1-indexed)
        assert!(content.contains("51"));
    }

    #[test]
    fn test_mode_change() {
        let mut renderer = LineNumberRenderer::with_default_mode();
        assert_eq!(renderer.mode(), LineNumberMode::Absolute);

        renderer.set_mode(LineNumberMode::Relative);
        assert_eq!(renderer.mode(), LineNumberMode::Relative);
    }

    #[test]
    fn test_zero_area() {
        let renderer = LineNumberRenderer::with_default_mode();
        let theme = builtin::gic_dark();

        let area = Rect::new(0, 0, 0, 0);
        let mut buf = Buffer::empty(Rect::new(0, 0, 1, 1));
        // Should not crash
        renderer.render(&mut buf, area, 0, 10, 0, &theme);
    }
}
