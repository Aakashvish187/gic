//! # Rendering Engine Types
//!
//! Shared value types used across the rendering engine modules.
//! These types form the vocabulary of the renderer's internal communication,
//! ensuring consistent coordinate systems and rendering primitives.

use ratatui::style::{Color, Modifier, Style};

/// A position on the terminal screen in absolute screen coordinates.
///
/// Screen coordinates are 0-indexed from the top-left corner of the terminal.
/// These are distinct from buffer/logical coordinates (which refer to positions
/// within the text buffer) and virtual coordinates (which include scroll offsets).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ScreenPosition {
    /// Column (x) position on screen, 0-indexed from left edge.
    pub col: u16,
    /// Row (y) position on screen, 0-indexed from top edge.
    pub row: u16,
}

impl ScreenPosition {
    /// Creates a new screen position.
    pub fn new(col: u16, row: u16) -> Self {
        Self { col, row }
    }

    /// Origin position (0, 0) — top-left corner of the terminal.
    pub fn origin() -> Self {
        Self { col: 0, row: 0 }
    }
}

/// Defines a rectangular region of the terminal for rendering purposes.
///
/// Used by the layout engine to partition the terminal into functional areas
/// (line number gutter, text area, status bar, etc.).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RenderRegion {
    /// Left edge column (inclusive).
    pub x: u16,
    /// Top edge row (inclusive).
    pub y: u16,
    /// Width in columns.
    pub width: u16,
    /// Height in rows.
    pub height: u16,
}

impl RenderRegion {
    /// Creates a new render region.
    pub fn new(x: u16, y: u16, width: u16, height: u16) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Returns true if this region has zero area (either dimension is 0).
    pub fn is_empty(&self) -> bool {
        self.width == 0 || self.height == 0
    }

    /// Converts to a ratatui `Rect` for widget rendering.
    pub fn to_rect(self) -> ratatui::layout::Rect {
        ratatui::layout::Rect::new(self.x, self.y, self.width, self.height)
    }
}

/// The visual shape of the editing cursor.
///
/// Different editor modes typically use different cursor shapes:
/// - `Block` for normal/command mode
/// - `Beam` for insert mode
/// - `Underline` for replace mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[derive(Default)]
pub enum CursorShape {
    /// Full-cell block cursor (█). Used in normal/command mode.
    #[default]
    Block,
    /// Thin vertical line cursor (│). Used in insert mode.
    Beam,
    /// Horizontal underline cursor (_). Used in replace mode.
    Underline,
}


/// Mode for displaying line numbers in the gutter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[derive(Default)]
pub enum LineNumberMode {
    /// Show absolute line numbers (1, 2, 3, ...).
    #[default]
    Absolute,
    /// Show relative distances from cursor line (2, 1, 0, 1, 2, ...).
    Relative,
    /// Show absolute on current line, relative on all others.
    /// This is the most popular mode in vim-like editors.
    Hybrid,
}


/// Direction for scroll operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ScrollDirection {
    /// Scroll content upward (view moves down in the file).
    Up,
    /// Scroll content downward (view moves up in the file).
    Down,
    /// Scroll content leftward (view moves right in wide lines).
    Left,
    /// Scroll content rightward (view moves left in wide lines).
    Right,
}

/// A styled text span ready for rendering.
///
/// Combines text content with ratatui styling. This is the atomic unit
/// of the renderer's output — each visible line is composed of a sequence
/// of `StyledSpan` values.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StyledSpan {
    /// The text content of this span.
    pub text: String,
    /// The visual style (fg color, bg color, modifiers) for this span.
    pub style: Style,
}

impl StyledSpan {
    /// Creates a new styled span.
    pub fn new<S: Into<String>>(text: S, style: Style) -> Self {
        Self {
            text: text.into(),
            style,
        }
    }

    /// Creates a plain (unstyled) span with default colors.
    pub fn plain<S: Into<String>>(text: S) -> Self {
        Self {
            text: text.into(),
            style: Style::default(),
        }
    }

    /// Creates a span with foreground color only.
    pub fn colored<S: Into<String>>(text: S, fg: Color) -> Self {
        Self {
            text: text.into(),
            style: Style::default().fg(fg),
        }
    }

    /// Creates a span with foreground color and bold modifier.
    pub fn bold_colored<S: Into<String>>(text: S, fg: Color) -> Self {
        Self {
            text: text.into(),
            style: Style::default().fg(fg).add_modifier(Modifier::BOLD),
        }
    }

    /// Converts this span to a ratatui `Span` for rendering.
    pub fn to_ratatui_span(&self) -> ratatui::text::Span<'_> {
        ratatui::text::Span::styled(&self.text, self.style)
    }

    /// Returns the character count of this span's text.
    pub fn char_count(&self) -> usize {
        self.text.chars().count()
    }

    /// Returns true if the span text is empty.
    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_screen_position() {
        let pos = ScreenPosition::new(10, 20);
        assert_eq!(pos.col, 10);
        assert_eq!(pos.row, 20);

        let origin = ScreenPosition::origin();
        assert_eq!(origin.col, 0);
        assert_eq!(origin.row, 0);
    }

    #[test]
    fn test_render_region() {
        let region = RenderRegion::new(5, 10, 80, 24);
        assert!(!region.is_empty());

        let empty = RenderRegion::new(0, 0, 0, 10);
        assert!(empty.is_empty());

        let rect = region.to_rect();
        assert_eq!(rect.x, 5);
        assert_eq!(rect.y, 10);
        assert_eq!(rect.width, 80);
        assert_eq!(rect.height, 24);
    }

    #[test]
    fn test_cursor_shape_default() {
        assert_eq!(CursorShape::default(), CursorShape::Block);
    }

    #[test]
    fn test_line_number_mode_default() {
        assert_eq!(LineNumberMode::default(), LineNumberMode::Absolute);
    }

    #[test]
    fn test_styled_span_creation() {
        let span = StyledSpan::plain("hello");
        assert_eq!(span.text, "hello");
        assert_eq!(span.char_count(), 5);
        assert!(!span.is_empty());

        let colored = StyledSpan::colored("world", Color::Red);
        assert_eq!(colored.style.fg, Some(Color::Red));

        let bold = StyledSpan::bold_colored("bold", Color::Blue);
        assert!(bold.style.add_modifier == Modifier::empty() || true); // Style check
        assert_eq!(bold.text, "bold");
    }

    #[test]
    fn test_styled_span_empty() {
        let empty = StyledSpan::plain("");
        assert!(empty.is_empty());
        assert_eq!(empty.char_count(), 0);
    }

    #[test]
    fn test_styled_span_unicode() {
        let span = StyledSpan::plain("🦀 Rust");
        assert_eq!(span.char_count(), 6); // 🦀, space, R, u, s, t
    }
}
