//! # Status Bar Renderer
//!
//! Full-featured status bar rendering with extensible widget system.
//! Displays file information, cursor position, editor mode, modification
//! state, and placeholder slots for future features (git, diagnostics).
//!
//! ## Design
//!
//! The status bar is divided into left-aligned and right-aligned sections:
//!
//! ```text
//! ┌──────────────────────────────────────────────────────────────┐
//! │ MODE │ filename [+] │ status    │    encoding LF Ln:Col 80x24│
//! └──────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Extension
//!
//! New segments can be added by modifying `build_left_spans` or
//! `build_right_spans` without touching the core rendering logic.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Widget;

use crate::renderer::dirty_indicator::DirtyIndicator;
use crate::renderer::file_info::FileInfo;
use crate::renderer::themes::Theme;

/// Production status bar renderer for the GIC editor.
///
/// Implements the ratatui `Widget` trait for direct rendering into
/// a ratatui frame.
pub struct StatusBarRenderer<'a> {
    /// File information snapshot.
    file_info: &'a FileInfo,
    /// Editor mode name (e.g., "NORMAL", "INSERT").
    mode: &'a str,
    /// Active theme for styling.
    theme: &'a Theme,
    /// Terminal dimensions (width, height).
    terminal_size: (u16, u16),
    /// Optional status message.
    status_message: Option<&'a str>,
    /// Optional git branch name.
    git_branch: Option<&'a str>,
    /// Optional error count.
    error_count: Option<usize>,
    /// Optional warning count.
    warning_count: Option<usize>,
}

impl<'a> StatusBarRenderer<'a> {
    /// Creates a new status bar renderer.
    pub fn new(
        file_info: &'a FileInfo,
        mode: &'a str,
        theme: &'a Theme,
        terminal_size: (u16, u16),
    ) -> Self {
        Self {
            file_info,
            mode,
            theme,
            terminal_size,
            status_message: None,
            git_branch: None,
            error_count: None,
            warning_count: None,
        }
    }

    /// Sets an optional status message.
    pub fn with_status_message(mut self, msg: &'a str) -> Self {
        self.status_message = Some(msg);
        self
    }

    /// Sets the git branch name.
    pub fn with_git_branch(mut self, branch: &'a str) -> Self {
        self.git_branch = Some(branch);
        self
    }

    /// Sets diagnostic counts.
    pub fn with_diagnostics(mut self, errors: usize, warnings: usize) -> Self {
        self.error_count = Some(errors);
        self.warning_count = Some(warnings);
        self
    }

    /// Builds the left-aligned spans of the status bar.
    fn build_left_spans(&self) -> Vec<Span<'static>> {
        let mut spans = Vec::new();

        // Mode indicator
        let mode_style = self.theme.status_bar_mode_style();
        spans.push(Span::styled(format!(" {} ", self.mode), mode_style));

        // Separator
        spans.push(Span::styled(" ", self.theme.status_bar_style()));

        // File name + dirty indicator
        let dirty_label =
            DirtyIndicator::short_label(self.file_info.is_modified, self.file_info.is_read_only);
        let file_display = if dirty_label.is_empty() {
            self.file_info.file_name.clone()
        } else {
            format!("{} {}", self.file_info.file_name, dirty_label)
        };
        spans.push(Span::styled(
            file_display,
            self.theme.status_bar_style().add_modifier(Modifier::BOLD),
        ));

        // Status message
        if let Some(msg) = self.status_message {
            spans.push(Span::styled(
                " │ ",
                Style::default().fg(self.theme.status_bar_secondary),
            ));
            spans.push(Span::styled(
                msg.to_string(),
                Style::default()
                    .fg(self.theme.status_bar_fg)
                    .bg(self.theme.status_bar_bg),
            ));
        }

        // Git branch (future)
        if let Some(branch) = self.git_branch {
            spans.push(Span::styled(
                " │ ",
                Style::default().fg(self.theme.status_bar_secondary),
            ));
            spans.push(Span::styled(
                format!(" {}", branch),
                Style::default()
                    .fg(self.theme.status_bar_fg)
                    .bg(self.theme.status_bar_bg),
            ));
        }

        // Diagnostics (future)
        if let Some(errors) = self.error_count {
            if errors > 0 {
                spans.push(Span::styled(
                    format!(" ✕{}", errors),
                    Style::default()
                        .fg(Color::Rgb(243, 139, 168))
                        .bg(self.theme.status_bar_bg),
                ));
            }
        }
        if let Some(warnings) = self.warning_count {
            if warnings > 0 {
                spans.push(Span::styled(
                    format!(" ⚠{}", warnings),
                    Style::default()
                        .fg(Color::Rgb(249, 226, 175))
                        .bg(self.theme.status_bar_bg),
                ));
            }
        }

        spans
    }

    /// Builds the right-aligned spans of the status bar.
    fn build_right_spans(&self) -> Vec<Span<'static>> {
        let mut spans = Vec::new();
        let secondary_style = Style::default()
            .fg(self.theme.status_bar_secondary)
            .bg(self.theme.status_bar_bg);
        let info_style = Style::default()
            .fg(self.theme.status_bar_fg)
            .bg(self.theme.status_bar_bg);

        // Language
        spans.push(Span::styled(
            format!("{} ", self.file_info.language),
            info_style,
        ));

        spans.push(Span::styled("│ ", secondary_style));

        // Encoding
        spans.push(Span::styled(
            format!("{} ", self.file_info.encoding),
            info_style,
        ));

        // Line ending
        spans.push(Span::styled(
            format!("{} ", self.file_info.line_ending),
            info_style,
        ));

        spans.push(Span::styled("│ ", secondary_style));

        // Cursor position
        spans.push(Span::styled(
            format!("{} ", self.file_info.cursor_display()),
            info_style,
        ));

        spans.push(Span::styled("│ ", secondary_style));

        // Terminal size
        spans.push(Span::styled(
            format!("{}×{} ", self.terminal_size.0, self.terminal_size.1),
            secondary_style,
        ));

        spans
    }
}

impl<'a> Widget for StatusBarRenderer<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.height == 0 || area.width == 0 {
            return;
        }

        // Fill background
        let bg_style = self.theme.status_bar_style();
        for x in area.x..area.x + area.width {
            for y in area.y..area.y + area.height {
                buf.get_mut(x, y).set_style(bg_style);
                buf.get_mut(x, y).set_symbol(" ");
            }
        }

        let left_spans = self.build_left_spans();
        let right_spans = self.build_right_spans();

        // Calculate widths
        let left_width: usize = left_spans.iter().map(|s| s.content.len()).sum();
        let right_width: usize = right_spans.iter().map(|s| s.content.len()).sum();
        let available = area.width as usize;

        // Render left-aligned spans
        let left_line = Line::from(left_spans);
        buf.set_line(area.x, area.y, &left_line, area.width);

        // Render right-aligned spans if there's room
        if left_width + right_width < available {
            let right_x = area.x + (available - right_width) as u16;
            let right_line = Line::from(right_spans);
            buf.set_line(right_x, area.y, &right_line, right_width as u16);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::renderer::themes::builtin;
    use gic_core::{Document, TextBuffer};

    fn make_test_file_info() -> FileInfo {
        let doc = Document::new_empty();
        let buffer = TextBuffer::from_str("Hello\nWorld");
        FileInfo::from_state(&doc, &buffer, 0, 5, Some("Rust"))
    }

    #[test]
    fn test_status_bar_render() {
        let file_info = make_test_file_info();
        let theme = builtin::gic_dark();
        let bar = StatusBarRenderer::new(&file_info, "NORMAL", &theme, (120, 40));

        let area = Rect::new(0, 0, 120, 1);
        let mut buf = Buffer::empty(area);
        bar.render(area, &mut buf);

        let content: String = buf
            .content()
            .iter()
            .map(|c| c.symbol().to_string())
            .collect();
        assert!(content.contains("NORMAL"));
        assert!(content.contains("[Untitled]"));
    }

    #[test]
    fn test_status_bar_with_message() {
        let file_info = make_test_file_info();
        let theme = builtin::gic_dark();
        let bar = StatusBarRenderer::new(&file_info, "INSERT", &theme, (80, 24))
            .with_status_message("File saved");

        let area = Rect::new(0, 0, 80, 1);
        let mut buf = Buffer::empty(area);
        bar.render(area, &mut buf);

        let content: String = buf
            .content()
            .iter()
            .map(|c| c.symbol().to_string())
            .collect();
        assert!(content.contains("INSERT"));
        assert!(content.contains("File saved"));
    }

    #[test]
    fn test_status_bar_with_diagnostics() {
        let file_info = make_test_file_info();
        let theme = builtin::gic_dark();
        let bar =
            StatusBarRenderer::new(&file_info, "NORMAL", &theme, (120, 40)).with_diagnostics(3, 7);

        let area = Rect::new(0, 0, 120, 1);
        let mut buf = Buffer::empty(area);
        bar.render(area, &mut buf);

        let content: String = buf
            .content()
            .iter()
            .map(|c| c.symbol().to_string())
            .collect();
        assert!(content.contains("✕3"));
        assert!(content.contains("⚠7"));
    }

    #[test]
    fn test_status_bar_narrow_terminal() {
        let file_info = make_test_file_info();
        let theme = builtin::gic_dark();
        let bar = StatusBarRenderer::new(&file_info, "NORMAL", &theme, (20, 10));

        let area = Rect::new(0, 0, 20, 1);
        let mut buf = Buffer::empty(area);
        // Should not crash on narrow terminal
        bar.render(area, &mut buf);
    }

    #[test]
    fn test_status_bar_zero_area() {
        let file_info = make_test_file_info();
        let theme = builtin::gic_dark();
        let bar = StatusBarRenderer::new(&file_info, "NORMAL", &theme, (0, 0));

        let area = Rect::new(0, 0, 0, 0);
        let mut buf = Buffer::empty(Rect::new(0, 0, 1, 1));
        // Should not crash
        bar.render(area, &mut buf);
    }
}
