//! # Layout Engine
//!
//! Partitions the terminal area into functional regions for the editor UI.
//! The layout engine computes the sizes and positions of:
//!
//! - **Line number gutter**: Dynamic width based on total line count.
//! - **Text editing area**: The main content area.
//! - **Status bar**: Fixed 1-row bar at the bottom.
//!
//! ## Future Extension
//!
//! The `EditorLayout` struct is designed to accommodate split panes, tabs,
//! breadcrumbs, and diagnostic panels by adding new `Rect` fields without
//! modifying the core layout calculation logic.

use ratatui::layout::Rect;

/// Computed layout regions for the editor UI.
///
/// Produced by [`LayoutEngine::compute`] from a terminal `Rect`.
/// All regions are non-overlapping and together cover the full terminal area.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditorLayout {
    /// The line number gutter area (left side).
    pub line_number_area: Rect,
    /// The main text editing area (center).
    pub text_area: Rect,
    /// The status bar area (bottom row).
    pub status_bar_area: Rect,
    /// Width of the line number gutter in columns.
    pub gutter_width: u16,
}

impl EditorLayout {
    /// Returns true if the text area has usable space for rendering.
    pub fn has_text_area(&self) -> bool {
        self.text_area.width > 0 && self.text_area.height > 0
    }

    /// Returns the number of text rows available for content.
    pub fn text_rows(&self) -> u16 {
        self.text_area.height
    }

    /// Returns the number of text columns available for content.
    pub fn text_cols(&self) -> u16 {
        self.text_area.width
    }
}

/// Computes editor layout from terminal dimensions and buffer state.
///
/// The layout engine is stateless — it takes inputs and produces an
/// [`EditorLayout`]. This makes it trivially testable and safe to call
/// on every frame without side effects.
pub struct LayoutEngine;

impl LayoutEngine {
    /// Computes the editor layout for the given terminal area and total line count.
    ///
    /// # Layout Strategy
    ///
    /// ```text
    /// ┌────────┬──────────────────────────────────┐
    /// │ Gutter │         Text Area                │
    /// │  (N)   │                                   │
    /// │        │                                   │
    /// ├────────┴──────────────────────────────────┤
    /// │              Status Bar (1 row)            │
    /// └───────────────────────────────────────────┘
    /// ```
    ///
    /// # Arguments
    ///
    /// * `area` - The full terminal area.
    /// * `total_lines` - Total number of lines in the buffer (determines gutter width).
    ///
    /// # Returns
    ///
    /// An [`EditorLayout`] with all regions computed, or a minimal layout if the
    /// terminal is too small.
    pub fn compute(area: Rect, total_lines: usize) -> EditorLayout {
        // Terminal too small — return degenerate layout
        if area.width < 3 || area.height < 2 {
            return EditorLayout {
                line_number_area: Rect::new(area.x, area.y, 0, 0),
                text_area: Rect::new(area.x, area.y, area.width, area.height.saturating_sub(1)),
                status_bar_area: Rect::new(
                    area.x,
                    area.y + area.height.saturating_sub(1),
                    area.width,
                    1.min(area.height),
                ),
                gutter_width: 0,
            };
        }

        let gutter_width = Self::calculate_gutter_width(total_lines);
        let status_bar_height: u16 = 1;
        let content_height = area.height.saturating_sub(status_bar_height);

        // Ensure gutter doesn't consume more than 1/4 of terminal width
        let effective_gutter = gutter_width.min(area.width / 4);
        let text_width = area.width.saturating_sub(effective_gutter);

        EditorLayout {
            line_number_area: Rect::new(area.x, area.y, effective_gutter, content_height),
            text_area: Rect::new(
                area.x + effective_gutter,
                area.y,
                text_width,
                content_height,
            ),
            status_bar_area: Rect::new(
                area.x,
                area.y + content_height,
                area.width,
                status_bar_height,
            ),
            gutter_width: effective_gutter,
        }
    }

    /// Calculates the width of the line number gutter.
    ///
    /// Width = number of digits in `total_lines` + 2 (1 space padding on each side).
    ///
    /// # Examples
    ///
    /// - 1-9 lines → 3 columns (` 1 `)
    /// - 10-99 lines → 4 columns (` 10 `)
    /// - 100-999 lines → 5 columns (` 100 `)
    /// - 10000+ lines → scales accordingly
    ///
    /// Minimum gutter width is 4 columns for visual consistency.
    pub fn calculate_gutter_width(total_lines: usize) -> u16 {
        let digits = if total_lines == 0 {
            1
        } else {
            (total_lines as f64).log10().floor() as u16 + 1
        };

        // digits + 2 padding, minimum 4
        (digits + 2).max(4)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gutter_width_small_file() {
        assert_eq!(LayoutEngine::calculate_gutter_width(1), 4); // min 4
        assert_eq!(LayoutEngine::calculate_gutter_width(9), 4); // 1 digit + 2 = 3, min 4
        assert_eq!(LayoutEngine::calculate_gutter_width(10), 4); // 2 digits + 2 = 4
        assert_eq!(LayoutEngine::calculate_gutter_width(99), 4); // 2 digits + 2 = 4
    }

    #[test]
    fn test_gutter_width_medium_file() {
        assert_eq!(LayoutEngine::calculate_gutter_width(100), 5); // 3 digits + 2
        assert_eq!(LayoutEngine::calculate_gutter_width(999), 5);
        assert_eq!(LayoutEngine::calculate_gutter_width(1000), 6); // 4 digits + 2
    }

    #[test]
    fn test_gutter_width_large_file() {
        assert_eq!(LayoutEngine::calculate_gutter_width(10_000), 7); // 5 digits + 2
        assert_eq!(LayoutEngine::calculate_gutter_width(100_000), 8); // 6 digits + 2
        assert_eq!(LayoutEngine::calculate_gutter_width(1_000_000), 9); // 7 digits + 2
    }

    #[test]
    fn test_gutter_width_zero_lines() {
        assert_eq!(LayoutEngine::calculate_gutter_width(0), 4); // min 4
    }

    #[test]
    fn test_layout_standard_terminal() {
        let area = Rect::new(0, 0, 120, 40);
        let layout = LayoutEngine::compute(area, 500);

        assert_eq!(layout.gutter_width, 5); // 3 digits + 2
        assert_eq!(layout.line_number_area.width, 5);
        assert_eq!(layout.text_area.x, 5);
        assert_eq!(layout.text_area.width, 115); // 120 - 5
        assert_eq!(layout.text_area.height, 39); // 40 - 1 status bar
        assert_eq!(layout.status_bar_area.y, 39);
        assert_eq!(layout.status_bar_area.height, 1);
        assert_eq!(layout.status_bar_area.width, 120);
    }

    #[test]
    fn test_layout_small_terminal() {
        let area = Rect::new(0, 0, 20, 5);
        let layout = LayoutEngine::compute(area, 50);

        assert!(layout.has_text_area());
        assert_eq!(layout.text_area.height, 4);
        assert_eq!(layout.status_bar_area.height, 1);
    }

    #[test]
    fn test_layout_tiny_terminal() {
        let area = Rect::new(0, 0, 2, 1);
        let layout = LayoutEngine::compute(area, 10);

        // Degenerate layout: no gutter, minimal text area
        assert_eq!(layout.gutter_width, 0);
        assert_eq!(layout.text_area.width, 2);
    }

    #[test]
    fn test_layout_gutter_cap() {
        // Very large file in narrow terminal — gutter should not exceed 1/4 width
        let area = Rect::new(0, 0, 20, 40);
        let layout = LayoutEngine::compute(area, 1_000_000);

        assert!(layout.gutter_width <= 5); // 20 / 4 = 5
        assert!(layout.text_area.width >= 15);
    }

    #[test]
    fn test_layout_has_text_area() {
        let area = Rect::new(0, 0, 80, 24);
        let layout = LayoutEngine::compute(area, 100);
        assert!(layout.has_text_area());

        let tiny = Rect::new(0, 0, 2, 1);
        let tiny_layout = LayoutEngine::compute(tiny, 10);
        // Even tiny terminals try to have some text area
        assert!(tiny_layout.text_area.width > 0 || tiny_layout.text_area.height == 0);
    }

    #[test]
    fn test_layout_text_rows_cols() {
        let area = Rect::new(0, 0, 100, 30);
        let layout = LayoutEngine::compute(area, 200);

        assert_eq!(layout.text_rows(), 29); // 30 - 1 status bar
        assert_eq!(layout.text_cols(), 100 - layout.gutter_width);
    }

    #[test]
    fn test_layout_regions_non_overlapping() {
        let area = Rect::new(0, 0, 120, 40);
        let layout = LayoutEngine::compute(area, 500);

        // Gutter and text area should not overlap horizontally
        assert_eq!(
            layout.line_number_area.x + layout.line_number_area.width,
            layout.text_area.x
        );

        // Content area and status bar should not overlap vertically
        assert_eq!(
            layout.text_area.y + layout.text_area.height,
            layout.status_bar_area.y
        );
    }
}
