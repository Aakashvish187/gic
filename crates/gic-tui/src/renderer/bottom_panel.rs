//! # Bottom Panel Renderer
//!
//! Terminal placeholder panel at the bottom of the IDE layout.
//! Shows tab bar and placeholder content (actual terminal integration is future).

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};

use crate::renderer::themes::Theme;

pub struct BottomPanelRenderer<'a> {
    pub active_tab: usize,
    pub theme: &'a Theme,
}

impl<'a> BottomPanelRenderer<'a> {
    pub fn new(theme: &'a Theme) -> Self {
        Self {
            active_tab: 0,
            theme,
        }
    }

    pub fn render(&self, area: Rect, buf: &mut Buffer) {
        if area.width < 10 || area.height < 3 {
            return;
        }

        let bg = Style::default()
            .fg(self.theme.foreground)
            .bg(self.theme.panel_bg);

        // Fill background
        for y in area.y..area.y + area.height {
            for x in area.x..area.x + area.width {
                buf.get_mut(x, y).set_style(bg).set_symbol(" ");
            }
        }

        // Top border
        let border_style = Style::default()
            .fg(self.theme.panel_border)
            .bg(self.theme.panel_bg);
        for x in area.x..area.x + area.width {
            buf.get_mut(x, area.y)
                .set_style(border_style)
                .set_symbol("─");
        }

        // Tab bar
        let tabs = ["Shell", "kubectl", "docker", "terraform", "git", "logs"];
        let mut spans = Vec::new();
        spans.push(Span::styled(" ", Style::default().bg(self.theme.panel_bg)));

        for (i, tab) in tabs.iter().enumerate() {
            let style = if i == self.active_tab {
                Style::default()
                    .fg(self.theme.top_bar_accent)
                    .bg(self.theme.panel_bg)
                    .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
            } else {
                Style::default()
                    .fg(self.theme.panel_header)
                    .bg(self.theme.panel_bg)
            };
            spans.push(Span::styled(format!(" {} ", tab), style));
            if i < tabs.len() - 1 {
                spans.push(Span::styled("│", border_style));
            }
        }

        if area.height > 1 {
            let tab_line = Line::from(spans);
            buf.set_line(area.x, area.y + 1, &tab_line, area.width);
        }

        // Content area - placeholder
        if area.height > 3 {
            let content_y = area.y + 3;
            let msg = Line::from(Span::styled(
                " $ Terminal integration coming soon...",
                Style::default()
                    .fg(self.theme.panel_border)
                    .bg(self.theme.panel_bg),
            ));
            buf.set_line(area.x, content_y, &msg, area.width);
        }
    }
}
