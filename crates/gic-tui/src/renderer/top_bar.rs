//! # Top Bar Renderer
//!
//! Renders the GIC branding bar at the top of the IDE layout:
//! `GIC │ filename │ FileType │ Git: branch │ Connected Cluster: none`

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Widget;

use crate::renderer::themes::Theme;

pub struct TopBarRenderer<'a> {
    pub filename: &'a str,
    pub file_type: &'a str,
    pub git_branch: Option<&'a str>,
    pub theme: &'a Theme,
}

impl<'a> TopBarRenderer<'a> {
    pub fn new(filename: &'a str, file_type: &'a str, theme: &'a Theme) -> Self {
        Self {
            filename,
            file_type,
            git_branch: None,
            theme,
        }
    }

    pub fn with_git_branch(mut self, branch: &'a str) -> Self {
        self.git_branch = Some(branch);
        self
    }
}

impl<'a> Widget for TopBarRenderer<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.height == 0 || area.width == 0 {
            return;
        }

        let bg = Style::default()
            .fg(self.theme.top_bar_fg)
            .bg(self.theme.top_bar_bg);
        // Fill background
        for x in area.x..area.x + area.width {
            buf.get_mut(x, area.y).set_style(bg).set_symbol(" ");
        }

        let mut spans = Vec::new();

        // Brand
        spans.push(Span::styled(
            " GIC ",
            Style::default()
                .fg(self.theme.top_bar_bg)
                .bg(self.theme.top_bar_accent)
                .add_modifier(Modifier::BOLD),
        ));

        let sep_style = Style::default()
            .fg(self.theme.panel_border)
            .bg(self.theme.top_bar_bg);
        spans.push(Span::styled(" │ ", sep_style));

        // Filename
        spans.push(Span::styled(
            self.filename.to_string(),
            Style::default()
                .fg(self.theme.top_bar_fg)
                .bg(self.theme.top_bar_bg)
                .add_modifier(Modifier::BOLD),
        ));

        spans.push(Span::styled(" │ ", sep_style));

        // File type
        spans.push(Span::styled(
            self.file_type.to_string(),
            Style::default()
                .fg(self.theme.top_bar_accent)
                .bg(self.theme.top_bar_bg),
        ));

        // Git branch
        if let Some(branch) = self.git_branch {
            spans.push(Span::styled(" │ ", sep_style));
            spans.push(Span::styled(
                format!("Git: {}", branch),
                Style::default()
                    .fg(Color::Rgb(166, 227, 161))
                    .bg(self.theme.top_bar_bg),
            ));
        }

        // Cluster (placeholder)
        spans.push(Span::styled(" │ ", sep_style));
        spans.push(Span::styled(
            "Cluster: none",
            Style::default()
                .fg(self.theme.panel_border)
                .bg(self.theme.top_bar_bg),
        ));

        let line = Line::from(spans);
        buf.set_line(area.x, area.y, &line, area.width);
    }
}
