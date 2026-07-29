//! # Viewport Engine
//!
//! Manages the visible window into the text buffer. The viewport tracks which
//! lines and columns are currently visible on screen, handles scroll offsets,
//! and provides coordinate translation between buffer space and screen space.
//!
//! ## Coordinate Systems
//!
//! - **Buffer coordinates**: (row, col) in the full text buffer (0-indexed).
//! - **Viewport coordinates**: (row, col) relative to the viewport origin.
//! - **Screen coordinates**: (x, y) absolute terminal positions.
//!
//! ## Performance
//!
//! All viewport operations are O(1) — pure integer arithmetic with no
//! allocations. This is critical for smooth scrolling at high frame rates.

/// Represents the visible window into a text buffer.
///
/// The viewport is a sliding window defined by scroll offsets and visible
/// dimensions. It never owns or references text data — it operates purely
/// on integer coordinates.
///
/// # Invariants
///
/// - `scroll_row + visible_rows <= total_lines` (clamped)
/// - `scroll_col` can exceed line lengths (horizontal scroll is unbounded
///   for display purposes; actual rendering clips at line end)
/// - `visible_rows` and `visible_cols` are always >= 1 when the viewport
///   is active
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Viewport {
    /// First visible buffer row (0-indexed).
    scroll_row: usize,
    /// First visible buffer column (0-indexed).
    scroll_col: usize,
    /// Number of rows visible in the viewport.
    visible_rows: usize,
    /// Number of columns visible in the viewport.
    visible_cols: usize,
    /// Total number of lines in the buffer.
    total_lines: usize,
}

impl Viewport {
    /// Creates a new viewport with the given dimensions.
    ///
    /// # Arguments
    ///
    /// * `visible_rows` - Height of the viewport in terminal rows.
    /// * `visible_cols` - Width of the viewport in terminal columns.
    /// * `total_lines` - Total number of lines in the text buffer.
    pub fn new(visible_rows: usize, visible_cols: usize, total_lines: usize) -> Self {
        Self {
            scroll_row: 0,
            scroll_col: 0,
            visible_rows: visible_rows.max(1),
            visible_cols: visible_cols.max(1),
            total_lines: total_lines.max(1),
        }
    }

    // ─── Accessors ───────────────────────────────────────────────────

    /// Returns the first visible row index.
    pub fn scroll_row(&self) -> usize {
        self.scroll_row
    }

    /// Returns the first visible column index.
    pub fn scroll_col(&self) -> usize {
        self.scroll_col
    }
    
    pub fn set_scroll_row(&mut self, row: usize) {
        self.scroll_row = row;
    }
    
    pub fn set_scroll_col(&mut self, col: usize) {
        self.scroll_col = col;
    }

    /// Returns the number of visible rows.
    pub fn visible_rows(&self) -> usize {
        self.visible_rows
    }

    /// Returns the number of visible columns.
    pub fn visible_cols(&self) -> usize {
        self.visible_cols
    }

    /// Returns total lines in the buffer.
    pub fn total_lines(&self) -> usize {
        self.total_lines
    }

    // ─── Dimension Updates ───────────────────────────────────────────

    /// Updates total line count (e.g., after buffer edit) and re-clamps scroll.
    pub fn set_total_lines(&mut self, total: usize) {
        self.total_lines = total.max(1);
        self.clamp();
    }

    /// Updates visible dimensions (e.g., after terminal resize) and re-clamps.
    pub fn resize(&mut self, visible_rows: usize, visible_cols: usize) {
        self.visible_rows = visible_rows.max(1);
        self.visible_cols = visible_cols.max(1);
        self.clamp();
    }

    // ─── Scroll Operations (all O(1)) ────────────────────────────────

    /// Scrolls up by `n` lines. Clamps at top of buffer.
    pub fn scroll_up(&mut self, n: usize) {
        self.scroll_row = self.scroll_row.saturating_sub(n);
    }

    /// Scrolls down by `n` lines. Clamps so last line remains visible.
    pub fn scroll_down(&mut self, n: usize) {
        self.scroll_row = self.scroll_row.saturating_add(n);
        self.clamp();
    }

    /// Scrolls left by `n` columns. Clamps at column 0.
    pub fn scroll_left(&mut self, n: usize) {
        self.scroll_col = self.scroll_col.saturating_sub(n);
    }

    /// Scrolls right by `n` columns.
    pub fn scroll_right(&mut self, n: usize) {
        self.scroll_col = self.scroll_col.saturating_add(n);
    }

    /// Scrolls up by one full page (viewport height).
    pub fn page_up(&mut self) {
        self.scroll_up(self.visible_rows.saturating_sub(1).max(1));
    }

    /// Scrolls down by one full page (viewport height).
    pub fn page_down(&mut self) {
        self.scroll_down(self.visible_rows.saturating_sub(1).max(1));
    }

    /// Scrolls to the very top of the buffer.
    pub fn scroll_to_top(&mut self) {
        self.scroll_row = 0;
        self.scroll_col = 0;
    }

    /// Scrolls to the very bottom of the buffer.
    pub fn scroll_to_bottom(&mut self) {
        self.scroll_row = self.total_lines.saturating_sub(self.visible_rows);
    }

    // ─── Cursor Following ────────────────────────────────────────────

    /// Adjusts scroll offsets to ensure the given cursor position is visible.
    ///
    /// Uses a context margin of `scroll_off` lines above/below the cursor
    /// to prevent the cursor from sitting at the very edge of the viewport.
    ///
    /// # Arguments
    ///
    /// * `cursor_row` - Buffer row of the cursor.
    /// * `cursor_col` - Buffer column of the cursor (display width).
    /// * `scroll_off` - Number of context lines to maintain around cursor.
    pub fn ensure_cursor_visible(
        &mut self,
        cursor_row: usize,
        cursor_col: usize,
        scroll_off: usize,
    ) {
        // Vertical adjustment
        let effective_scroll_off = scroll_off.min(self.visible_rows / 2);

        if cursor_row < self.scroll_row + effective_scroll_off {
            self.scroll_row = cursor_row.saturating_sub(effective_scroll_off);
        } else if cursor_row >= self.scroll_row + self.visible_rows - effective_scroll_off {
            self.scroll_row = cursor_row
                .saturating_sub(self.visible_rows.saturating_sub(1 + effective_scroll_off));
        }

        // Horizontal adjustment
        let h_margin = 8_usize.min(self.visible_cols / 4);
        if cursor_col < self.scroll_col {
            self.scroll_col = cursor_col.saturating_sub(h_margin);
        } else if cursor_col >= self.scroll_col + self.visible_cols {
            self.scroll_col =
                cursor_col.saturating_sub(self.visible_cols.saturating_sub(1 + h_margin));
        }

        self.clamp();
    }

    /// Scrolls to center the given row in the viewport.
    pub fn center_on_row(&mut self, row: usize) {
        let half = self.visible_rows / 2;
        self.scroll_row = row.saturating_sub(half);
        self.clamp();
    }

    // ─── Query Methods ───────────────────────────────────────────────

    /// Returns the range of visible buffer rows as `(start, end)` exclusive.
    ///
    /// The returned range is clamped to `[0, total_lines)`.
    pub fn visible_line_range(&self) -> (usize, usize) {
        let start = self.scroll_row;
        let end = (self.scroll_row + self.visible_rows).min(self.total_lines);
        (start, end)
    }

    /// Returns the range of visible columns as `(start, end)` exclusive.
    pub fn visible_col_range(&self) -> (usize, usize) {
        let start = self.scroll_col;
        let end = self.scroll_col + self.visible_cols;
        (start, end)
    }

    /// Returns true if the given buffer row is currently visible.
    pub fn is_line_visible(&self, row: usize) -> bool {
        let (start, end) = self.visible_line_range();
        row >= start && row < end
    }

    /// Converts a buffer row to a viewport-relative row.
    ///
    /// Returns `None` if the row is not visible.
    pub fn buffer_row_to_viewport(&self, buffer_row: usize) -> Option<usize> {
        if self.is_line_visible(buffer_row) {
            Some(buffer_row - self.scroll_row)
        } else {
            None
        }
    }

    /// Converts a viewport-relative row to a buffer row.
    pub fn viewport_row_to_buffer(&self, viewport_row: usize) -> usize {
        self.scroll_row + viewport_row
    }

    /// Converts a buffer column to a viewport-relative column.
    ///
    /// Returns `None` if the column is scrolled out of view to the left.
    pub fn buffer_col_to_viewport(&self, buffer_col: usize) -> Option<usize> {
        if buffer_col >= self.scroll_col {
            let viewport_col = buffer_col - self.scroll_col;
            if viewport_col < self.visible_cols {
                Some(viewport_col)
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Returns the number of visible lines (may be less than `visible_rows`
    /// near the end of the buffer).
    pub fn visible_line_count(&self) -> usize {
        let (start, end) = self.visible_line_range();
        end - start
    }

    // ─── Internal ────────────────────────────────────────────────────

    /// Clamps scroll_row so the viewport doesn't extend past the buffer.
    fn clamp(&mut self) {
        let max_scroll = self.total_lines.saturating_sub(self.visible_rows);
        if self.scroll_row > max_scroll {
            self.scroll_row = max_scroll;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_viewport_new() {
        let vp = Viewport::new(24, 80, 100);
        assert_eq!(vp.scroll_row(), 0);
        assert_eq!(vp.scroll_col(), 0);
        assert_eq!(vp.visible_rows(), 24);
        assert_eq!(vp.visible_cols(), 80);
        assert_eq!(vp.total_lines(), 100);
    }

    #[test]
    fn test_viewport_minimum_dimensions() {
        let vp = Viewport::new(0, 0, 0);
        assert_eq!(vp.visible_rows(), 1);
        assert_eq!(vp.visible_cols(), 1);
        assert_eq!(vp.total_lines(), 1);
    }

    #[test]
    fn test_scroll_down_clamp() {
        let mut vp = Viewport::new(10, 80, 20);
        vp.scroll_down(15);
        assert_eq!(vp.scroll_row(), 10); // 20 - 10 = max scroll of 10
    }

    #[test]
    fn test_scroll_up_clamp() {
        let mut vp = Viewport::new(10, 80, 100);
        vp.scroll_down(5);
        vp.scroll_up(10); // Should clamp at 0
        assert_eq!(vp.scroll_row(), 0);
    }

    #[test]
    fn test_page_up_down() {
        let mut vp = Viewport::new(10, 80, 100);
        vp.page_down(); // scroll by 9 (visible_rows - 1)
        assert_eq!(vp.scroll_row(), 9);

        vp.page_down();
        assert_eq!(vp.scroll_row(), 18);

        vp.page_up();
        assert_eq!(vp.scroll_row(), 9);
    }

    #[test]
    fn test_scroll_to_top_bottom() {
        let mut vp = Viewport::new(10, 80, 100);
        vp.scroll_to_bottom();
        assert_eq!(vp.scroll_row(), 90);

        vp.scroll_to_top();
        assert_eq!(vp.scroll_row(), 0);
        assert_eq!(vp.scroll_col(), 0);
    }

    #[test]
    fn test_visible_line_range() {
        let mut vp = Viewport::new(10, 80, 100);
        assert_eq!(vp.visible_line_range(), (0, 10));

        vp.scroll_down(5);
        assert_eq!(vp.visible_line_range(), (5, 15));

        // Near end of buffer
        vp.scroll_down(90);
        assert_eq!(vp.visible_line_range(), (90, 100));
    }

    #[test]
    fn test_is_line_visible() {
        let mut vp = Viewport::new(10, 80, 100);
        assert!(vp.is_line_visible(0));
        assert!(vp.is_line_visible(9));
        assert!(!vp.is_line_visible(10));

        vp.scroll_down(5);
        assert!(!vp.is_line_visible(4));
        assert!(vp.is_line_visible(5));
        assert!(vp.is_line_visible(14));
        assert!(!vp.is_line_visible(15));
    }

    #[test]
    fn test_buffer_to_viewport_conversion() {
        let mut vp = Viewport::new(10, 80, 100);
        vp.scroll_down(5);

        assert_eq!(vp.buffer_row_to_viewport(5), Some(0));
        assert_eq!(vp.buffer_row_to_viewport(14), Some(9));
        assert_eq!(vp.buffer_row_to_viewport(4), None);
        assert_eq!(vp.buffer_row_to_viewport(15), None);
    }

    #[test]
    fn test_viewport_to_buffer_conversion() {
        let mut vp = Viewport::new(10, 80, 100);
        vp.scroll_down(5);

        assert_eq!(vp.viewport_row_to_buffer(0), 5);
        assert_eq!(vp.viewport_row_to_buffer(9), 14);
    }

    #[test]
    fn test_horizontal_scroll() {
        let mut vp = Viewport::new(10, 80, 100);
        vp.scroll_right(20);
        assert_eq!(vp.scroll_col(), 20);
        assert_eq!(vp.visible_col_range(), (20, 100));

        vp.scroll_left(10);
        assert_eq!(vp.scroll_col(), 10);

        vp.scroll_left(100); // Clamp at 0
        assert_eq!(vp.scroll_col(), 0);
    }

    #[test]
    fn test_ensure_cursor_visible_vertical() {
        let mut vp = Viewport::new(10, 80, 100);

        // Cursor below viewport
        vp.ensure_cursor_visible(15, 0, 3);
        assert!(vp.is_line_visible(15));

        // Cursor above viewport
        vp.scroll_down(50);
        vp.ensure_cursor_visible(20, 0, 3);
        assert!(vp.is_line_visible(20));
    }

    #[test]
    fn test_ensure_cursor_visible_horizontal() {
        let mut vp = Viewport::new(10, 40, 100);

        // Cursor beyond right edge
        vp.ensure_cursor_visible(0, 50, 3);
        assert!(vp.buffer_col_to_viewport(50).is_some());
    }

    #[test]
    fn test_resize() {
        let mut vp = Viewport::new(10, 80, 100);
        vp.scroll_down(95);
        assert_eq!(vp.scroll_row(), 90); // max scroll = 100 - 10

        vp.resize(20, 120);
        assert_eq!(vp.visible_rows(), 20);
        assert_eq!(vp.visible_cols(), 120);
        assert_eq!(vp.scroll_row(), 80); // re-clamped: max scroll = 100 - 20
    }

    #[test]
    fn test_center_on_row() {
        let mut vp = Viewport::new(10, 80, 100);
        vp.center_on_row(50);
        assert_eq!(vp.scroll_row(), 45); // 50 - 10/2 = 45
        assert!(vp.is_line_visible(50));
    }

    #[test]
    fn test_set_total_lines() {
        let mut vp = Viewport::new(10, 80, 100);
        vp.scroll_down(95);
        assert_eq!(vp.scroll_row(), 90);

        // Shrink buffer
        vp.set_total_lines(50);
        assert_eq!(vp.scroll_row(), 40); // re-clamped: 50 - 10
    }

    #[test]
    fn test_visible_line_count() {
        let mut vp = Viewport::new(10, 80, 5);
        assert_eq!(vp.visible_line_count(), 5); // Only 5 lines exist

        vp.set_total_lines(100);
        assert_eq!(vp.visible_line_count(), 10);
    }

    #[test]
    fn test_large_file_viewport() {
        let mut vp = Viewport::new(50, 200, 1_000_000);
        vp.scroll_to_bottom();
        assert_eq!(vp.scroll_row(), 999_950);

        vp.ensure_cursor_visible(500_000, 0, 5);
        assert!(vp.is_line_visible(500_000));
    }

    #[test]
    fn test_buffer_col_to_viewport() {
        let mut vp = Viewport::new(10, 40, 100);
        vp.scroll_right(10);

        assert_eq!(vp.buffer_col_to_viewport(10), Some(0));
        assert_eq!(vp.buffer_col_to_viewport(49), Some(39));
        assert_eq!(vp.buffer_col_to_viewport(9), None); // scrolled past
        assert_eq!(vp.buffer_col_to_viewport(50), None); // beyond visible
    }
}
