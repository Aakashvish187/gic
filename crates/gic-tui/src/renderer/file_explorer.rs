//! # File Explorer Renderer
//!
//! Renders a tree-view of the project directory in the left panel.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};

use crate::renderer::themes::Theme;

/// A single entry in the file explorer tree.
#[derive(Debug, Clone)]
pub struct FileTreeEntry {
    pub name: String,
    pub is_dir: bool,
    pub depth: usize,
    pub expanded: bool,
    pub is_active: bool,
}

impl FileTreeEntry {
    pub fn icon(&self) -> &'static str {
        if self.is_dir {
            if self.expanded { "📂" } else { "📁" }
        } else {
            let name_lower = self.name.to_lowercase();
            if name_lower.contains("dockerfile") || name_lower == "containerfile" {
                return "🐳";
            }
            if name_lower == "values.yaml" || name_lower == "chart.yaml" {
                return "⎈";
            }
            if name_lower == "playbook.yml" || name_lower == "ansible.cfg" {
                return "⚙";
            }
            if name_lower == "build.yml" || name_lower == "deploy.yml" || name_lower == "ci.yml" {
                return "🚀";
            }
            if name_lower == "jenkinsfile" {
                return "🔧";
            }
            if name_lower == "makefile" {
                return "⚡";
            }

            match self.name.rsplit('.').next().unwrap_or("") {
                "yaml" | "yml" => "☸", // Default to K8s for YAML
                "tf" | "tfvars" => "🌎",
                "sh" | "bash" => "🐚",
                "rs" => "🦀",
                "json" => "📋",
                "toml" => "⚙",
                "md" => "📝",
                "py" => "🐍",
                "go" => "🔵",
                "js" | "ts" => "📜",
                _ => "📄",
            }
        }
    }
}

/// Renders the file explorer panel.
pub struct FileExplorerRenderer<'a> {
    pub entries: &'a [FileTreeEntry],
    pub selected_index: usize,
    pub scroll_offset: usize,
    pub theme: &'a Theme,
    pub title: &'a str,
}

impl<'a> FileExplorerRenderer<'a> {
    pub fn new(entries: &'a [FileTreeEntry], theme: &'a Theme) -> Self {
        Self {
            entries,
            selected_index: 0,
            scroll_offset: 0,
            theme,
            title: "EXPLORER",
        }
    }

    pub fn with_selection(mut self, index: usize) -> Self {
        self.selected_index = index;
        self
    }

    pub fn with_scroll(mut self, offset: usize) -> Self {
        self.scroll_offset = offset;
        self
    }

    pub fn render(&self, area: Rect, buf: &mut Buffer) {
        if area.width < 4 || area.height < 2 {
            return;
        }

        let bg = Style::default().fg(self.theme.panel_header).bg(self.theme.panel_bg);

        // Fill background
        for y in area.y..area.y + area.height {
            for x in area.x..area.x + area.width {
                buf.get_mut(x, y).set_style(bg).set_symbol(" ");
            }
        }

        // Draw right border
        let border_style = Style::default().fg(self.theme.panel_border).bg(self.theme.panel_bg);
        let border_x = area.x + area.width - 1;
        for y in area.y..area.y + area.height {
            buf.get_mut(border_x, y).set_style(border_style).set_symbol("│");
        }

        // Header
        let header = Line::from(vec![
            Span::styled(
                format!(" {} ", self.title),
                Style::default()
                    .fg(self.theme.panel_header)
                    .bg(self.theme.panel_bg)
                    .add_modifier(Modifier::BOLD),
            ),
        ]);
        buf.set_line(area.x, area.y, &header, area.width.saturating_sub(1));

        // Separator line
        if area.height > 1 {
            let sep = "─".repeat((area.width.saturating_sub(2)) as usize);
            let sep_line = Line::from(Span::styled(
                format!(" {}", sep),
                border_style,
            ));
            buf.set_line(area.x, area.y + 1, &sep_line, area.width.saturating_sub(1));
        }

        // Tree entries
        let content_start = area.y + 2;
        let content_height = area.height.saturating_sub(2) as usize;
        let usable_width = area.width.saturating_sub(2) as usize; // 1 padding + 1 border

        for (i, entry) in self.entries.iter().skip(self.scroll_offset).take(content_height).enumerate() {
            let y = content_start + i as u16;
            if y >= area.y + area.height {
                break;
            }

            let global_idx = self.scroll_offset + i;
            let is_selected = global_idx == self.selected_index;

            // Background for selected item
            if is_selected {
                let sel_style = Style::default().bg(self.theme.explorer_active);
                for x in area.x..border_x {
                    buf.get_mut(x, y).set_style(sel_style);
                }
            }

            let indent = "  ".repeat(entry.depth);
            let icon = entry.icon();
            let display = format!(" {}{} {}", indent, icon, entry.name);

            // Truncate to fit
            let truncated: String = display.chars().take(usable_width).collect();

            let style = if is_selected {
                Style::default()
                    .fg(self.theme.foreground)
                    .bg(self.theme.explorer_active)
                    .add_modifier(Modifier::BOLD)
            } else if entry.is_active {
                Style::default()
                    .fg(self.theme.top_bar_accent)
                    .bg(self.theme.panel_bg)
            } else if entry.is_dir {
                Style::default()
                    .fg(self.theme.panel_header)
                    .bg(self.theme.panel_bg)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
                    .fg(self.theme.foreground)
                    .bg(self.theme.panel_bg)
            };

            let line = Line::from(Span::styled(truncated, style));
            buf.set_line(area.x, y, &line, area.width.saturating_sub(1));
        }
    }
}
