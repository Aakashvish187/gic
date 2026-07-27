//! # Cursor Renderer
//!
//! Computes screen-space cursor position from buffer coordinates and viewport
//! offsets. Handles cursor shape (Block/Beam/Underline), visibility clipping,
//! and color from the active theme.
//!
//! The cursor renderer never modifies state — it reads buffer position and
//! viewport, then computes where to place the terminal cursor.

use crate::renderer::text_renderer::TextRenderer;
use crate::renderer::types::{CursorShape, ScreenPosition};
use crate::renderer::viewport::Viewport;

use gic_core::CursorPosition;
use ratatui::layout::Rect;

/// Computed cursor rendering information for a single frame.
///
/// This struct contains everything needed to place and style the cursor
/// on the terminal. It is produced by [`CursorRenderer::compute`] and
/// consumed by the render pipeline.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CursorRenderInfo {
    /// Screen position where the cursor should be rendered.
    pub screen_position: ScreenPosition,
    /// Visual shape of the cursor.
    pub shape: CursorShape,
    /// Whether the cursor is visible (within the viewport).
    pub visible: bool,
}

/// Computes cursor screen position and visibility.
///
/// The cursor renderer is stateless — it takes the current cursor position,
/// viewport state, and layout information, and produces a [`CursorRenderInfo`].
pub struct CursorRenderer {
    /// Cursor shape for the current editor mode.
    shape: CursorShape,
    /// Text renderer for display width calculation.
    text_renderer: TextRenderer,
}

impl CursorRenderer {
    /// Creates a new cursor renderer with the given shape.
    pub fn new(shape: CursorShape, tab_width: usize) -> Self {
        Self {
            shape,
            text_renderer: TextRenderer::new(tab_width),
        }
    }

    /// Creates a cursor renderer with default settings (Block cursor, tab width 4).
    pub fn with_defaults() -> Self {
        Self::new(CursorShape::Block, 4)
    }

    /// Updates the cursor shape (e.g., when switching between Normal and Insert mode).
    pub fn set_shape(&mut self, shape: CursorShape) {
        self.shape = shape;
    }

    /// Returns the current cursor shape.
    pub fn shape(&self) -> CursorShape {
        self.shape
    }

    /// Computes cursor rendering information for the current frame.
    ///
    /// # Arguments
    ///
    /// * `cursor_pos` - Buffer position of the cursor (row, col).
    /// * `viewport` - Current viewport state.
    /// * `text_area` - The `Rect` defining the text editing area on screen.
    /// * `line_text` - The text of the line the cursor is on (for width calculation).
    ///
    /// # Returns
    ///
    /// A [`CursorRenderInfo`] with computed screen position and visibility.
    pub fn compute(
        &self,
        cursor_pos: CursorPosition,
        viewport: &Viewport,
        text_area: Rect,
        line_text: Option<&str>,
    ) -> CursorRenderInfo {
        // Check vertical visibility
        let viewport_row = match viewport.buffer_row_to_viewport(cursor_pos.row) {
            Some(r) => r,
            None => {
                return CursorRenderInfo {
                    screen_position: ScreenPosition::origin(),
                    shape: self.shape,
                    visible: false,
                };
            }
        };

        // Calculate display column from character position
        let display_col = match line_text {
            Some(text) => self
                .text_renderer
                .char_index_to_display_col(text, cursor_pos.col),
            None => cursor_pos.col,
        };

        // Check horizontal visibility
        let viewport_col = match viewport.buffer_col_to_viewport(display_col) {
            Some(c) => c,
            None => {
                // Cursor is scrolled out of view horizontally
                // If cursor is to the left of viewport, place at left edge
                if display_col < viewport.scroll_col() {
                    return CursorRenderInfo {
                        screen_position: ScreenPosition::new(
                            text_area.x,
                            text_area.y + viewport_row as u16,
                        ),
                        shape: self.shape,
                        visible: false,
                    };
                }
                return CursorRenderInfo {
                    screen_position: ScreenPosition::origin(),
                    shape: self.shape,
                    visible: false,
                };
            }
        };

        // Convert to screen coordinates
        let screen_col = text_area.x + viewport_col as u16;
        let screen_row = text_area.y + viewport_row as u16;

        // Clamp to text area bounds
        let visible = screen_col < text_area.x + text_area.width
            && screen_row < text_area.y + text_area.height;

        CursorRenderInfo {
            screen_position: ScreenPosition::new(screen_col, screen_row),
            shape: self.shape,
            visible,
        }
    }

    /// Returns the crossterm cursor style command for the current shape.
    ///
    /// This is used by the pipeline to set the terminal cursor style
    /// after rendering the frame.
    pub fn crossterm_cursor_style(&self) -> crossterm::cursor::SetCursorStyle {
        match self.shape {
            CursorShape::Block => crossterm::cursor::SetCursorStyle::SteadyBlock,
            CursorShape::Beam => crossterm::cursor::SetCursorStyle::SteadyBar,
            CursorShape::Underline => crossterm::cursor::SetCursorStyle::SteadyUnderScore,
        }
    }
}

/// Maps an editor mode name to the appropriate cursor shape.
///
/// This provides a conventional mapping:
/// - "NORMAL" / "VISUAL" → Block
/// - "INSERT" → Beam
/// - "REPLACE" → Underline
pub fn cursor_shape_for_mode(mode: &str) -> CursorShape {
    match mode.to_uppercase().as_str() {
        "INSERT" => CursorShape::Beam,
        "REPLACE" => CursorShape::Underline,
        _ => CursorShape::Block,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cursor_renderer_defaults() {
        let cr = CursorRenderer::with_defaults();
        assert_eq!(cr.shape(), CursorShape::Block);
    }

    #[test]
    fn test_cursor_set_shape() {
        let mut cr = CursorRenderer::with_defaults();
        cr.set_shape(CursorShape::Beam);
        assert_eq!(cr.shape(), CursorShape::Beam);
    }

    #[test]
    fn test_cursor_compute_visible() {
        let cr = CursorRenderer::with_defaults();
        let viewport = Viewport::new(24, 80, 100);
        let text_area = Rect::new(5, 0, 75, 23);
        let cursor_pos = CursorPosition::new(5, 10);

        let info = cr.compute(cursor_pos, &viewport, text_area, Some("Hello World!"));

        assert!(info.visible);
        assert_eq!(info.screen_position.row, 5);
        assert_eq!(info.screen_position.col, 15); // text_area.x(5) + col(10)
    }

    #[test]
    fn test_cursor_compute_invisible_below_viewport() {
        let cr = CursorRenderer::with_defaults();
        let viewport = Viewport::new(10, 80, 100);
        let text_area = Rect::new(5, 0, 75, 9);
        let cursor_pos = CursorPosition::new(20, 0); // Below viewport

        let info = cr.compute(cursor_pos, &viewport, text_area, Some(""));
        assert!(!info.visible);
    }

    #[test]
    fn test_cursor_compute_with_tabs() {
        let cr = CursorRenderer::new(CursorShape::Block, 4);
        let viewport = Viewport::new(24, 80, 100);
        let text_area = Rect::new(5, 0, 75, 23);

        // Cursor after a tab character: "\t" occupies 4 display columns
        let cursor_pos = CursorPosition::new(0, 1);
        let info = cr.compute(cursor_pos, &viewport, text_area, Some("\thello"));

        assert!(info.visible);
        // Display col 4 (after tab) + text_area.x(5) = 9
        assert_eq!(info.screen_position.col, 9);
    }

    #[test]
    fn test_cursor_compute_with_horizontal_scroll() {
        let cr = CursorRenderer::with_defaults();
        let mut viewport = Viewport::new(24, 40, 100);
        viewport.scroll_right(10);
        let text_area = Rect::new(5, 0, 35, 23);

        // Cursor at display col 15, scroll_col = 10 → viewport col = 5
        let cursor_pos = CursorPosition::new(0, 15);
        let info = cr.compute(
            cursor_pos,
            &viewport,
            text_area,
            Some("abcdefghijklmnopqrstuvwxyz"),
        );

        assert!(info.visible);
        assert_eq!(info.screen_position.col, 10); // text_area.x(5) + viewport_col(5)
    }

    #[test]
    fn test_cursor_shape_for_mode() {
        assert_eq!(cursor_shape_for_mode("NORMAL"), CursorShape::Block);
        assert_eq!(cursor_shape_for_mode("INSERT"), CursorShape::Beam);
        assert_eq!(cursor_shape_for_mode("REPLACE"), CursorShape::Underline);
        assert_eq!(cursor_shape_for_mode("VISUAL"), CursorShape::Block);
        assert_eq!(cursor_shape_for_mode("insert"), CursorShape::Beam); // case insensitive
    }
}
