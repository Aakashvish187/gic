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
    /// Optional debug metrics string (FPS, Draw Calls).
    debug_metrics: Option<String>,
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
            debug_metrics: None,
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

    /// Sets debug metrics string.
    pub fn with_debug_metrics(mut self, metrics: String) -> Self {
        self.debug_metrics = Some(metrics);
        self
    }

    /// Builds the left-aligned spans of the status bar.
    fn build_left_spans(&self) -> Vec<Span<'static>> {
        let mut spans = Vec::new();
        let sep_style = Style::default()
            .fg(self.theme.status_bar_secondary)
            .bg(self.theme.status_bar_bg);
        let info_style = Style::default()
            .fg(self.theme.status_bar_fg)
            .bg(self.theme.status_bar_bg);

        // 1. Mode indicator
        let mode_style = self.theme.status_bar_mode_style();
        spans.push(Span::styled(format!(" {} ", self.mode), mode_style));

        // Separator
        spans.push(Span::styled(" │ ", sep_style));

        // 2. Language
        let lang = if self.file_info.language == "Plain Text" {
            "TXT"
        } else {
            &self.file_info.language
        };
        spans.push(Span::styled(lang.to_string(), info_style));
        spans.push(Span::styled(" │ ", sep_style));

        // 3. Engine (Heuristic)
        let engine = if lang.to_lowercase().contains("yaml") {
            "Kubernetes"
        } else if lang.to_lowercase().contains("docker") {
            "Docker"
        } else if lang.to_lowercase().contains("terraform") {
            "Terraform"
        } else {
            "Standard"
        };
        spans.push(Span::styled(engine.to_string(), info_style));
        spans.push(Span::styled(" │ ", sep_style));

        // 4. Encoding
        spans.push(Span::styled(
            self.file_info.encoding.to_string(),
            info_style,
        ));
        spans.push(Span::styled(" │ ", sep_style));

        // 5. Indentation
        spans.push(Span::styled("Spaces:4", info_style)); // Hardcoded for now
        spans.push(Span::styled(" │ ", sep_style));

        // 6. Git branch
        let branch = self.git_branch.unwrap_or("main");
        spans.push(Span::styled(format!("Git:{}", branch), info_style));
        spans.push(Span::styled(" │ ", sep_style));

        // Status message overlay
        if let Some(msg) = self.status_message {
            spans.push(Span::styled(msg.to_string(), info_style));
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

        // 7. Validity / Diagnostics
        let errors = self.error_count.unwrap_or(0);
        let warnings = self.warning_count.unwrap_or(0);

        if errors == 0 && warnings == 0 {
            spans.push(Span::styled(
                " ☸ Valid ",
                Style::default()
                    .fg(Color::Rgb(166, 227, 161))
                    .bg(self.theme.status_bar_bg),
            ));
        } else {
            if warnings > 0 {
                spans.push(Span::styled(
                    format!(" ⚠{} ", warnings),
                    Style::default()
                        .fg(self.theme.diagnostic_warning)
                        .bg(self.theme.status_bar_bg),
                ));
            }
            if errors > 0 {
                spans.push(Span::styled(
                    format!(" ❌{} ", errors),
                    Style::default()
                        .fg(self.theme.diagnostic_error)
                        .bg(self.theme.status_bar_bg),
                ));
            }
        }

        spans.push(Span::styled("│ ", secondary_style));

        // Cursor position
        spans.push(Span::styled(
            format!("{} ", self.file_info.cursor_display()),
            info_style,
        ));

        // Debug metrics
        if let Some(ref metrics) = self.debug_metrics {
            spans.push(Span::styled("│ ", secondary_style));
            spans.push(Span::styled(
                format!("{} ", metrics),
                Style::default()
                    .fg(Color::Yellow)
                    .bg(self.theme.status_bar_bg),
            ));
        }

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
        assert!(content.contains("Rust"));
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
        assert!(content.contains("3"));
        assert!(content.contains("7"));
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
