//! # Intelligence Panel Renderer
//!
//! Right-side panel showing diagnostics, security, best practices,
//! documentation, and quick fixes for the current file.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};

use crate::renderer::themes::Theme;
use gic_core::language_engine::{EngineDiagnostic, EngineSeverity, HoverInfo};

pub struct IntelligencePanelRenderer<'a> {
    pub diagnostics: &'a [EngineDiagnostic],
    pub hover_info: Option<&'a HoverInfo>,
    pub theme: &'a Theme,
    pub scroll_offset: usize,
}

impl<'a> IntelligencePanelRenderer<'a> {
    pub fn new(diagnostics: &'a [EngineDiagnostic], theme: &'a Theme) -> Self {
        Self {
            diagnostics,
            hover_info: None,
            theme,
            scroll_offset: 0,
        }
    }

    pub fn with_hover(mut self, info: &'a HoverInfo) -> Self {
        self.hover_info = Some(info);
        self
    }

    pub fn render(&self, area: Rect, buf: &mut Buffer) {
        if area.width < 6 || area.height < 3 {
            return;
        }

        let bg = Style::default()
            .fg(self.theme.panel_header)
            .bg(self.theme.panel_bg);

        // Fill background
        for y in area.y..area.y + area.height {
            for x in area.x..area.x + area.width {
                buf.get_mut(x, y).set_style(bg).set_symbol(" ");
            }
        }

        // Draw left border
        let border_style = Style::default()
            .fg(self.theme.panel_border)
            .bg(self.theme.panel_bg);
        for y in area.y..area.y + area.height {
            buf.get_mut(area.x, y)
                .set_style(border_style)
                .set_symbol("│");
        }

        let content_x = area.x + 2; // after border + 1 padding
        let content_width = area.width.saturating_sub(3) as usize;
        let mut current_y = area.y;

        // Title
        let title = Line::from(Span::styled(
            " INTELLIGENCE",
            Style::default()
                .fg(self.theme.panel_header)
                .bg(self.theme.panel_bg)
                .add_modifier(Modifier::BOLD),
        ));
        buf.set_line(area.x + 1, current_y, &title, area.width.saturating_sub(1));
        current_y += 1;

        // Separator
        let sep = "─".repeat(content_width.min(area.width.saturating_sub(2) as usize));
        let sep_line = Line::from(Span::styled(sep, border_style));
        buf.set_line(
            content_x,
            current_y,
            &sep_line,
            area.width.saturating_sub(3),
        );
        current_y += 1;

        // Diagnostics section
        if !self.diagnostics.is_empty() {
            let error_count = self
                .diagnostics
                .iter()
                .filter(|d| d.severity == EngineSeverity::Error)
                .count();
            let warn_count = self
                .diagnostics
                .iter()
                .filter(|d| d.severity == EngineSeverity::Warning)
                .count();
            let hint_count = self
                .diagnostics
                .iter()
                .filter(|d| {
                    d.severity == EngineSeverity::Hint || d.severity == EngineSeverity::Info
                })
                .count();

            // Section header
            let header = Line::from(vec![
                Span::styled(
                    "DIAGNOSTICS",
                    Style::default()
                        .fg(self.theme.panel_header)
                        .bg(self.theme.panel_bg)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!(" ❌{} ⚠{} 💡{}", error_count, warn_count, hint_count),
                    Style::default()
                        .fg(self.theme.foreground)
                        .bg(self.theme.panel_bg),
                ),
            ]);
            if current_y < area.y + area.height {
                buf.set_line(content_x, current_y, &header, content_width as u16);
                current_y += 1;
            }

            // List diagnostics
            for diag in self.diagnostics.iter().take(10) {
                if current_y >= area.y + area.height {
                    break;
                }

                let severity_color = match diag.severity {
                    EngineSeverity::Error => self.theme.diagnostic_error,
                    EngineSeverity::Warning => self.theme.diagnostic_warning,
                    EngineSeverity::Info | EngineSeverity::Hint => self.theme.diagnostic_info,
                };

                let icon = diag.severity.icon();
                let code_str = diag.code.as_deref().unwrap_or("");
                let msg: String = format!("{} Ln{}: {}", icon, diag.row + 1, diag.message)
                    .chars()
                    .take(content_width)
                    .collect();

                let line = Line::from(Span::styled(
                    msg,
                    Style::default().fg(severity_color).bg(self.theme.panel_bg),
                ));
                buf.set_line(content_x, current_y, &line, content_width as u16);
                current_y += 1;
            }

            current_y += 1; // spacing
        }

        // Documentation section (from hover)
        if let Some(hover) = self.hover_info {
            if current_y < area.y + area.height {
                let header = Line::from(Span::styled(
                    "DOCUMENTATION",
                    Style::default()
                        .fg(self.theme.panel_header)
                        .bg(self.theme.panel_bg)
                        .add_modifier(Modifier::BOLD),
                ));
                buf.set_line(content_x, current_y, &header, content_width as u16);
                current_y += 1;
            }

            // Title
            if current_y < area.y + area.height {
                let title_line = Line::from(Span::styled(
                    truncate_str(&hover.title, content_width),
                    Style::default()
                        .fg(self.theme.top_bar_accent)
                        .bg(self.theme.panel_bg)
                        .add_modifier(Modifier::BOLD),
                ));
                buf.set_line(content_x, current_y, &title_line, content_width as u16);
                current_y += 1;
            }

            // Description (word-wrapped)
            for desc_line in wrap_text(&hover.description, content_width) {
                if current_y >= area.y + area.height {
                    break;
                }
                let line = Line::from(Span::styled(
                    desc_line,
                    Style::default()
                        .fg(self.theme.foreground)
                        .bg(self.theme.panel_bg),
                ));
                buf.set_line(content_x, current_y, &line, content_width as u16);
                current_y += 1;
            }

            // Best practice
            if let Some(bp) = &hover.best_practice {
                current_y += 1;
                if current_y < area.y + area.height {
                    let bp_header = Line::from(Span::styled(
                        "💡 Best Practice:",
                        Style::default()
                            .fg(self.theme.diagnostic_info)
                            .bg(self.theme.panel_bg)
                            .add_modifier(Modifier::BOLD),
                    ));
                    buf.set_line(content_x, current_y, &bp_header, content_width as u16);
                    current_y += 1;
                }
                for bp_line in wrap_text(bp, content_width) {
                    if current_y >= area.y + area.height {
                        break;
                    }
                    let line = Line::from(Span::styled(
                        bp_line,
                        Style::default()
                            .fg(self.theme.diagnostic_info)
                            .bg(self.theme.panel_bg),
                    ));
                    buf.set_line(content_x, current_y, &line, content_width as u16);
                    current_y += 1;
                }
            }
        }

        // If nothing to show
        if self.diagnostics.is_empty()
            && self.hover_info.is_none()
            && current_y < area.y + area.height
        {
            let msg = Line::from(Span::styled(
                "No issues found ✓",
                Style::default()
                    .fg(self.theme.panel_border)
                    .bg(self.theme.panel_bg),
            ));
            buf.set_line(content_x, current_y, &msg, content_width as u16);
        }
    }
}

fn truncate_str(s: &str, max_len: usize) -> String {
    s.chars().take(max_len).collect()
}

fn wrap_text(text: &str, width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current_line = String::new();

    for word in text.split_whitespace() {
        if current_line.is_empty() {
            current_line = word.to_string();
        } else if current_line.len() + 1 + word.len() <= width {
            current_line.push(' ');
            current_line.push_str(word);
        } else {
            lines.push(current_line);
            current_line = word.to_string();
        }
    }
    if !current_line.is_empty() {
        lines.push(current_line);
    }
    lines
}
