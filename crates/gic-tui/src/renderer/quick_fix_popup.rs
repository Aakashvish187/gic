use crate::renderer::themes::Theme;
use gic_core::language_engine::EngineQuickFix;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Widget},
};

pub struct QuickFixPopup<'a> {
    fixes: &'a [EngineQuickFix],
    selected_index: usize,
    theme: &'a Theme,
    screen_col: u16,
    screen_row: u16,
}

impl<'a> QuickFixPopup<'a> {
    pub fn new(
        fixes: &'a [EngineQuickFix],
        selected_index: usize,
        theme: &'a Theme,
        screen_col: u16,
        screen_row: u16,
    ) -> Self {
        Self {
            fixes,
            selected_index,
            theme,
            screen_col,
            screen_row,
        }
    }

    pub fn render(self, area: Rect, buf: &mut Buffer) {
        if self.fixes.is_empty() {
            return;
        }

        let width = 50;
        let height = self.fixes.len() as u16 + 2; // +2 for borders

        // Determine if popup should drop down or pop up
        let mut row = self.screen_row + 1;
        if row + height > area.bottom() && self.screen_row > height {
            row = self.screen_row.saturating_sub(height);
        }

        let mut col = self.screen_col;
        if col + width > area.right() {
            col = area.right().saturating_sub(width);
        }

        let popup_area = Rect::new(col, row, width, height);

        Clear.render(popup_area, buf);

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow))
            .style(Style::default().bg(self.theme.background))
            .title(" Quick Fixes ");

        let inner_area = block.inner(popup_area);
        block.render(popup_area, buf);

        let mut items = Vec::new();
        for (i, fix) in self.fixes.iter().enumerate() {
            let is_selected = i == self.selected_index;

            let mut style = Style::default().fg(self.theme.foreground);
            if is_selected {
                style = style.bg(self.theme.selection).add_modifier(Modifier::BOLD);
            }

            let icon = if fix.is_preferred { "★ " } else { "  " };
            let icon_style = Style::default().fg(Color::Yellow);

            let label_span = Span::styled(fix.title.clone(), style);

            let line = Line::from(vec![Span::styled(icon, icon_style), label_span]);

            items.push(ListItem::new(line).style(style));
        }

        let list = List::new(items);
        list.render(inner_area, buf);
    }
}
