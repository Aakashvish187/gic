use crate::renderer::themes::Theme;
use gic_core::language_engine::Completion;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Widget},
};

pub struct AutocompletePopup<'a> {
    completions: &'a [Completion],
    selected_index: usize,
    scroll_offset: usize,
    theme: &'a Theme,
    screen_col: u16,
    screen_row: u16,
    max_height: u16,
}

impl<'a> AutocompletePopup<'a> {
    pub fn new(
        completions: &'a [Completion],
        selected_index: usize,
        scroll_offset: usize,
        theme: &'a Theme,
        screen_col: u16,
        screen_row: u16,
        max_height: u16,
    ) -> Self {
        Self {
            completions,
            selected_index,
            scroll_offset,
            theme,
            screen_col,
            screen_row,
            max_height,
        }
    }

    pub fn render(self, area: Rect, buf: &mut Buffer) {
        if self.completions.is_empty() {
            return;
        }

        let width = 40;
        let height = (self.completions.len() as u16 + 2).min(self.max_height); // +2 for borders

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
            .border_style(Style::default().fg(Color::DarkGray))
            .style(Style::default().bg(self.theme.background));

        let inner_area = block.inner(popup_area);
        block.render(popup_area, buf);

        let visible_items = self
            .completions
            .iter()
            .skip(self.scroll_offset)
            .take(inner_area.height as usize);

        let mut items = Vec::new();
        for (i, comp) in visible_items.enumerate() {
            let actual_idx = i + self.scroll_offset;
            let is_selected = actual_idx == self.selected_index;

            let icon_str = comp.kind.icon();

            let mut style = Style::default().fg(self.theme.foreground);
            if is_selected {
                style = style.bg(self.theme.selection).add_modifier(Modifier::BOLD);
            }

            let icon_style = Style::default()
                .fg(self.theme.syntax.keyword)
                .add_modifier(Modifier::BOLD);

            let label_span = Span::styled(comp.label.clone(), style);
            let detail_span = Span::styled(
                format!(" {}", comp.detail.as_deref().unwrap_or("")),
                Style::default().fg(self.theme.syntax.comment),
            );

            let line = Line::from(vec![
                Span::styled(format!(" {} ", icon_str), icon_style),
                label_span,
                detail_span,
            ]);

            items.push(ListItem::new(line).style(style));
        }

        let list = List::new(items);
        list.render(inner_area, buf);
    }
}
