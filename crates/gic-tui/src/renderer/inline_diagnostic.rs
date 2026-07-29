//! # Inline Diagnostic Renderer
//!
//! Renders diagnostic messages directly below the affected line.
//! Example:
//! ```text
//!   apiVersion: apps/v1
//!   kind: UnknownResource
//!         ^ Unknown Kubernetes resource kind: 'UnknownResource'
//! ```

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};

use crate::renderer::themes::Theme;
use gic_core::language_engine::{EngineDiagnostic, EngineSeverity};

pub struct InlineDiagnosticRenderer<'a> {
    pub diagnostic: &'a EngineDiagnostic,
    pub theme: &'a Theme,
}

impl<'a> InlineDiagnosticRenderer<'a> {
    pub fn new(diagnostic: &'a EngineDiagnostic, theme: &'a Theme) -> Self {
        Self { diagnostic, theme }
    }

    pub fn render(&self, area: Rect, buf: &mut Buffer) {
        if area.width < 10 || area.height == 0 {
            return;
        }

        let severity_color = match self.diagnostic.severity {
            EngineSeverity::Error => self.theme.diagnostic_error,
            EngineSeverity::Warning => self.theme.diagnostic_warning,
            EngineSeverity::Info | EngineSeverity::Hint => self.theme.diagnostic_info,
        };

        let style = Style::default()
            .fg(severity_color)
            .bg(self.theme.background);

        // Fill background
        for x in area.x..area.x + area.width {
            buf.get_mut(x, area.y)
                .set_style(Style::default().bg(self.theme.background))
                .set_symbol(" ");
        }

        let mut spans = Vec::new();

        // Indent to the column where the error starts
        let indent = " ".repeat(self.diagnostic.col.min(area.width as usize));
        spans.push(Span::styled(indent, style));

        // Caret pointing to error
        spans.push(Span::styled("└─ ", style));

        // Icon + Message
        let icon = self.diagnostic.severity.icon();
        let msg = format!("{} {}", icon, self.diagnostic.message);

        // Ensure it fits
        let remaining_width = area.width.saturating_sub(self.diagnostic.col as u16 + 3);
        let truncated: String = msg.chars().take(remaining_width as usize).collect();

        spans.push(Span::styled(truncated, style));

        let line = Line::from(spans);
        buf.set_line(area.x, area.y, &line, area.width);
    }
}
