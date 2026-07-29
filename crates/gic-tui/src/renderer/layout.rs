//! # Layout Engine
//!
//! Partitions the terminal area into functional regions for the IDE layout.
//! Supports top bar, file explorer, editor panes, intelligence panel,
//! bottom panel, and status bar.

use ratatui::layout::Rect;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaneLayout {
    pub pane_index: usize,
    pub full_area: Rect,
    pub tab_bar_area: Option<Rect>,
    pub line_number_area: Rect,
    pub text_area: Rect,
    pub gutter_width: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditorLayout {
    pub top_bar_area: Rect,
    pub status_bar_area: Rect,
    pub file_explorer_area: Option<Rect>,
    pub intelligence_panel_area: Option<Rect>,
    pub bottom_panel_area: Option<Rect>,
    pub command_palette_area: Option<Rect>,
    pub panes: Vec<PaneLayout>,
}

pub struct LayoutEngine;

impl LayoutEngine {
    pub fn compute(
        area: Rect,
        file_explorer_open: bool,
        intelligence_panel_open: bool,
        bottom_panel_open: bool,
        command_palette_open: bool,
        pane_line_counts: &[usize],
    ) -> EditorLayout {
        if area.width < 10 || area.height < 5 {
            return EditorLayout {
                top_bar_area: Rect::new(area.x, area.y, area.width, 1),
                status_bar_area: Rect::new(
                    area.x,
                    area.y.saturating_add(area.height).saturating_sub(1),
                    area.width,
                    1,
                ),
                file_explorer_area: None,
                intelligence_panel_area: None,
                bottom_panel_area: None,
                command_palette_area: None,
                panes: Vec::new(),
            };
        }

        let mut current_area = area;

        // 1. Command Palette Overlay (centered, absolute positioning)
        let command_palette_area = if command_palette_open {
            let width = (area.width * 60) / 100;
            let height = (area.height * 40) / 100;
            let x = area.x + (area.width.saturating_sub(width)) / 2;
            let y = area.y + (area.height.saturating_sub(height)) / 4; // higher up
            Some(Rect::new(x, y, width, height.max(3)))
        } else {
            None
        };

        // 2. Top Bar (top 1 row)
        let top_bar_area = Rect::new(current_area.x, current_area.y, current_area.width, 1);
        current_area.y += 1;
        current_area.height = current_area.height.saturating_sub(1);

        // 3. Status Bar (bottom 1 row)
        let status_bar_area = Rect::new(
            current_area.x,
            current_area.y + current_area.height - 1,
            current_area.width,
            1,
        );
        current_area.height = current_area.height.saturating_sub(1);

        // 4. Bottom Panel (above status bar)
        let bottom_panel_area = if bottom_panel_open && current_area.height > 15 {
            let panel_height = 8.min(current_area.height / 3);
            let rect = Rect::new(
                current_area.x,
                current_area.y + current_area.height - panel_height,
                current_area.width,
                panel_height,
            );
            current_area.height = current_area.height.saturating_sub(panel_height);
            Some(rect)
        } else {
            None
        };

        // 5. File Explorer (left side)
        let file_explorer_area = if file_explorer_open && current_area.width > 40 {
            let width = 25.min(current_area.width / 4);
            let rect = Rect::new(current_area.x, current_area.y, width, current_area.height);
            current_area.x += width;
            current_area.width = current_area.width.saturating_sub(width);
            Some(rect)
        } else {
            None
        };

        // 6. Intelligence Panel (right side)
        let intelligence_panel_area = if intelligence_panel_open && current_area.width > 60 {
            let width = 30.min(current_area.width / 3);
            let rect = Rect::new(
                current_area.x + current_area.width - width,
                current_area.y,
                width,
                current_area.height,
            );
            current_area.width = current_area.width.saturating_sub(width);
            Some(rect)
        } else {
            None
        };

        // 7. Split Panes (remaining area)
        let mut panes = Vec::new();
        let pane_count = pane_line_counts.len().max(1);
        let pane_width = current_area.width / (pane_count as u16);

        for (i, &total_lines) in pane_line_counts.iter().enumerate() {
            let is_last = i == pane_count - 1;
            let width = if is_last {
                current_area.width - (pane_width * (i as u16))
            } else {
                pane_width
            };
            let pane_area = Rect::new(
                current_area.x + pane_width * (i as u16),
                current_area.y,
                width,
                current_area.height,
            );

            // Tab bar (top 1 row of each pane)
            let tab_bar_area = Rect::new(pane_area.x, pane_area.y, pane_area.width, 1);
            let mut content_area = pane_area;
            content_area.y += 1;
            content_area.height = content_area.height.saturating_sub(1);

            let gutter_width = Self::calculate_gutter_width(total_lines);
            let line_number_width = gutter_width + 1;

            let line_number_area = Rect::new(
                content_area.x,
                content_area.y,
                line_number_width.min(content_area.width),
                content_area.height,
            );
            let text_x = content_area.x + line_number_area.width;
            let text_width = content_area.width.saturating_sub(line_number_area.width);
            let text_area = Rect::new(text_x, content_area.y, text_width, content_area.height);

            panes.push(PaneLayout {
                pane_index: i,
                full_area: pane_area,
                tab_bar_area: Some(tab_bar_area),
                line_number_area,
                text_area,
                gutter_width,
            });
        }

        EditorLayout {
            top_bar_area,
            status_bar_area,
            file_explorer_area,
            intelligence_panel_area,
            bottom_panel_area,
            command_palette_area,
            panes,
        }
    }

    pub fn calculate_gutter_width(total_lines: usize) -> u16 {
        let max_digits = if total_lines == 0 {
            1
        } else {
            (total_lines as f64).log10().floor() as u16 + 1
        };
        max_digits.max(2)
    }
}
