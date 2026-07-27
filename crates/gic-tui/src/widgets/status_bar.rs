use gic_core::EngineState;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Widget,
};

/// Modular status bar UI component.
pub struct StatusBar<'a> {
    state: &'a EngineState,
}

impl<'a> StatusBar<'a> {
    pub fn new(state: &'a EngineState) -> Self {
        Self { state }
    }
}

impl<'a> Widget for StatusBar<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.height == 0 || area.width == 0 {
            return;
        }

        let mode_style = Style::default()
            .bg(Color::Cyan)
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD);

        let mouse_style = if self.state.mouse_enabled {
            Style::default().bg(Color::Green).fg(Color::Black)
        } else {
            Style::default().bg(Color::Yellow).fg(Color::Black)
        };

        let metrics_text = format!(
            "FPS: {:.1} | Ticks: {} | {}x{} ",
            self.state.metrics.current_fps,
            self.state.metrics.tick_count,
            self.state.metrics.screen_width,
            self.state.metrics.screen_height
        );

        let line = Line::from(vec![
            Span::styled(format!(" {} ", self.state.active_mode), mode_style),
            Span::styled(
                format!(
                    " MOUSE: {} ",
                    if self.state.mouse_enabled {
                        "ON"
                    } else {
                        "OFF"
                    }
                ),
                mouse_style,
            ),
            Span::styled(" ", Style::default()),
            Span::styled(
                &self.state.status_message,
                Style::default().fg(Color::White),
            ),
            Span::styled(
                format!("{:>width$}", metrics_text, width = area.width as usize),
                Style::default().fg(Color::DarkGray),
            ),
        ]);

        buf.set_line(area.x, area.y, &line, area.width);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_bar_render_buffer() {
        let mut state = EngineState::new();
        state.set_status("Test Running");
        let bar = StatusBar::new(&state);

        let area = Rect::new(0, 0, 80, 1);
        let mut buf = Buffer::empty(area);

        bar.render(area, &mut buf);

        // Check if buffer contains status text
        let content: String = buf.content().iter().map(|c| c.symbol()).collect();
        assert!(content.contains("NORMAL"));
        assert!(content.contains("MOUSE: ON"));
        assert!(content.contains("Test Running"));
    }
}
