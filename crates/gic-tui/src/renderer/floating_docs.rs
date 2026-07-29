use crate::renderer::themes::Theme;
use gic_core::language_engine::HoverInfo;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Widget, Wrap};

pub struct FloatingDocsRenderer<'a> {
    hover_info: &'a HoverInfo,
    theme: &'a Theme,
    cursor_x: u16,
    cursor_y: u16,
}

impl<'a> FloatingDocsRenderer<'a> {
    pub fn new(hover_info: &'a HoverInfo, theme: &'a Theme, cursor_x: u16, cursor_y: u16) -> Self {
        Self {
            hover_info,
            theme,
            cursor_x,
            cursor_y,
        }
    }

    pub fn render(self, area: Rect, buf: &mut Buffer) {
        let max_width = 60.min(area.width.saturating_sub(4));
        if max_width < 10 {
            return;
        }

        let mut lines = Vec::new();

        // Title
        lines.push(Line::from(vec![
            Span::styled(" 💡 ", Style::default().fg(Color::Yellow)),
            Span::styled(
                self.hover_info.title.clone(),
                Style::default()
                    .fg(self.theme.foreground)
                    .add_modifier(Modifier::BOLD),
            ),
        ]));

        lines.push(Line::from("")); // Spacer

        // Description
        let desc_lines: Vec<&str> = self.hover_info.description.lines().collect();
        for d in desc_lines {
            lines.push(Line::from(Span::styled(
                d,
                Style::default().fg(Color::Gray),
            )));
        }

        // Syntax
        if let Some(syntax) = &self.hover_info.syntax {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                " Syntax:",
                Style::default().fg(Color::Cyan),
            )));
            for s in syntax.lines() {
                lines.push(Line::from(Span::styled(
                    format!("  {}", s),
                    Style::default().fg(Color::White),
                )));
            }
        }

        // Best Practice
        if let Some(practice) = &self.hover_info.best_practice {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                " Best Practice:",
                Style::default().fg(Color::Green),
            )));
            for p in practice.lines() {
                lines.push(Line::from(Span::styled(
                    format!("  {}", p),
                    Style::default().fg(Color::White),
                )));
            }
        }

        let content_height = lines.len() as u16;
        let box_height = content_height + 2; // + borders
        let box_width = max_width;

        // Position: Try above cursor, if not enough space, try below
        let mut popup_y = self.cursor_y.saturating_sub(box_height);
        if popup_y < area.y || popup_y > area.y + area.height {
            popup_y = self.cursor_y.saturating_add(1);
        }

        // Keep inside screen bounds vertically
        if popup_y + box_height > area.y + area.height {
            popup_y = (area.y + area.height).saturating_sub(box_height);
        }

        // Position horizontally near cursor
        let mut popup_x = self.cursor_x;
        if popup_x + box_width > area.x + area.width {
            popup_x = (area.x + area.width).saturating_sub(box_width);
        }

        let popup_area = Rect::new(popup_x, popup_y, box_width, box_height);

        // Render shadow or just clear
        Clear.render(popup_area, buf);

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.panel_border))
            .style(Style::default().bg(self.theme.panel_bg));

        let paragraph = Paragraph::new(lines).block(block).wrap(Wrap { trim: true });

        ratatui::widgets::Widget::render(paragraph, popup_area, buf);
    }
}
