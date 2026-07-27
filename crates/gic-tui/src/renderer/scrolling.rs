//! # Scroll Controller
//!
//! Manages scroll behavior including context margins, cursor following,
//! and smooth scrolling state. The scroll controller drives viewport
//! updates without owning the viewport — it computes scroll adjustments
//! that are applied to a [`Viewport`].
//!
//! ## Context Margins (`scroll_off`)
//!
//! The controller maintains a configurable number of context lines above
//! and below the cursor, preventing the cursor from sitting at the very
//! edge of the viewport. This matches vim's `scrolloff` behavior.

use crate::renderer::viewport::Viewport;

/// Default number of context lines to keep above/below the cursor.
const DEFAULT_SCROLL_OFF: usize = 5;

/// Default number of lines to scroll per mouse wheel tick.
const DEFAULT_SCROLL_SPEED: usize = 3;

/// Controls scrolling behavior and cursor-following logic.
///
/// The scroll controller is configured once and then used to process
/// scroll events. It does not own the viewport — it takes `&mut Viewport`
/// and modifies it in place.
#[derive(Debug, Clone)]
pub struct ScrollController {
    /// Number of context lines to maintain above/below cursor.
    scroll_off: usize,
    /// Number of lines to scroll per mouse wheel event.
    scroll_speed: usize,
    /// Horizontal scroll speed in columns.
    horizontal_scroll_speed: usize,
}

impl Default for ScrollController {
    fn default() -> Self {
        Self::new()
    }
}

impl ScrollController {
    /// Creates a new scroll controller with default settings.
    pub fn new() -> Self {
        Self {
            scroll_off: DEFAULT_SCROLL_OFF,
            scroll_speed: DEFAULT_SCROLL_SPEED,
            horizontal_scroll_speed: 4,
        }
    }

    /// Creates a scroll controller with custom scroll-off margin.
    pub fn with_scroll_off(scroll_off: usize) -> Self {
        Self {
            scroll_off,
            ..Self::new()
        }
    }

    /// Returns the current scroll-off setting.
    pub fn scroll_off(&self) -> usize {
        self.scroll_off
    }

    /// Sets the scroll-off margin.
    pub fn set_scroll_off(&mut self, scroll_off: usize) {
        self.scroll_off = scroll_off;
    }

    /// Returns the scroll speed (lines per wheel tick).
    pub fn scroll_speed(&self) -> usize {
        self.scroll_speed
    }

    /// Sets the scroll speed.
    pub fn set_scroll_speed(&mut self, speed: usize) {
        self.scroll_speed = speed.max(1);
    }

    // ─── Cursor Following ────────────────────────────────────────────

    /// Adjusts the viewport to ensure the cursor is visible with context margins.
    ///
    /// This is the primary method called after any cursor movement to keep
    /// the cursor within the viewport bounds.
    ///
    /// # Arguments
    ///
    /// * `viewport` - Mutable reference to the viewport to adjust.
    /// * `cursor_row` - Buffer row of the cursor.
    /// * `cursor_display_col` - Display column of the cursor (after tab expansion).
    pub fn follow_cursor(
        &self,
        viewport: &mut Viewport,
        cursor_row: usize,
        cursor_display_col: usize,
    ) {
        viewport.ensure_cursor_visible(cursor_row, cursor_display_col, self.scroll_off);
    }

    // ─── Directional Scroll ──────────────────────────────────────────

    /// Scrolls the viewport up by the configured scroll speed.
    pub fn scroll_up(&self, viewport: &mut Viewport) {
        viewport.scroll_up(self.scroll_speed);
    }

    /// Scrolls the viewport down by the configured scroll speed.
    pub fn scroll_down(&self, viewport: &mut Viewport) {
        viewport.scroll_down(self.scroll_speed);
    }

    /// Scrolls the viewport left by the configured horizontal speed.
    pub fn scroll_left(&self, viewport: &mut Viewport) {
        viewport.scroll_left(self.horizontal_scroll_speed);
    }

    /// Scrolls the viewport right by the configured horizontal speed.
    pub fn scroll_right(&self, viewport: &mut Viewport) {
        viewport.scroll_right(self.horizontal_scroll_speed);
    }

    /// Scrolls up by one full page.
    pub fn page_up(&self, viewport: &mut Viewport) {
        viewport.page_up();
    }

    /// Scrolls down by one full page.
    pub fn page_down(&self, viewport: &mut Viewport) {
        viewport.page_down();
    }

    /// Scrolls to the top of the buffer.
    pub fn scroll_to_top(&self, viewport: &mut Viewport) {
        viewport.scroll_to_top();
    }

    /// Scrolls to the bottom of the buffer.
    pub fn scroll_to_bottom(&self, viewport: &mut Viewport) {
        viewport.scroll_to_bottom();
    }

    /// Centers the viewport on the given buffer row.
    pub fn center_on_row(&self, viewport: &mut Viewport, row: usize) {
        viewport.center_on_row(row);
    }

    /// Scrolls up by exactly `n` lines.
    pub fn scroll_up_n(&self, viewport: &mut Viewport, n: usize) {
        viewport.scroll_up(n);
    }

    /// Scrolls down by exactly `n` lines.
    pub fn scroll_down_n(&self, viewport: &mut Viewport, n: usize) {
        viewport.scroll_down(n);
    }

    // ─── Resize Handling ─────────────────────────────────────────────

    /// Handles a terminal resize event by updating viewport dimensions
    /// and re-following the cursor.
    pub fn handle_resize(
        &self,
        viewport: &mut Viewport,
        new_rows: usize,
        new_cols: usize,
        cursor_row: usize,
        cursor_display_col: usize,
    ) {
        viewport.resize(new_rows, new_cols);
        self.follow_cursor(viewport, cursor_row, cursor_display_col);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scroll_controller_defaults() {
        let sc = ScrollController::new();
        assert_eq!(sc.scroll_off(), DEFAULT_SCROLL_OFF);
        assert_eq!(sc.scroll_speed(), DEFAULT_SCROLL_SPEED);
    }

    #[test]
    fn test_scroll_controller_custom_scroll_off() {
        let sc = ScrollController::with_scroll_off(10);
        assert_eq!(sc.scroll_off(), 10);
    }

    #[test]
    fn test_scroll_up_down() {
        let sc = ScrollController::new();
        let mut vp = Viewport::new(20, 80, 100);

        sc.scroll_down(&mut vp);
        assert_eq!(vp.scroll_row(), 3); // DEFAULT_SCROLL_SPEED = 3

        sc.scroll_down(&mut vp);
        assert_eq!(vp.scroll_row(), 6);

        sc.scroll_up(&mut vp);
        assert_eq!(vp.scroll_row(), 3);
    }

    #[test]
    fn test_page_up_down() {
        let sc = ScrollController::new();
        let mut vp = Viewport::new(20, 80, 100);

        sc.page_down(&mut vp);
        assert_eq!(vp.scroll_row(), 19); // 20 - 1

        sc.page_up(&mut vp);
        assert_eq!(vp.scroll_row(), 0);
    }

    #[test]
    fn test_follow_cursor_down() {
        let sc = ScrollController::with_scroll_off(3);
        let mut vp = Viewport::new(20, 80, 100);

        // Cursor moves to row 20 (beyond visible range 0..20)
        sc.follow_cursor(&mut vp, 20, 0);
        assert!(vp.is_line_visible(20));
    }

    #[test]
    fn test_follow_cursor_up() {
        let sc = ScrollController::with_scroll_off(3);
        let mut vp = Viewport::new(20, 80, 100);
        vp.scroll_down(50);

        // Cursor moves to row 40 (above visible range 50..70)
        sc.follow_cursor(&mut vp, 40, 0);
        assert!(vp.is_line_visible(40));
    }

    #[test]
    fn test_handle_resize() {
        let sc = ScrollController::new();
        let mut vp = Viewport::new(20, 80, 100);
        vp.scroll_down(50);

        sc.handle_resize(&mut vp, 40, 120, 55, 0);
        assert_eq!(vp.visible_rows(), 40);
        assert_eq!(vp.visible_cols(), 120);
        assert!(vp.is_line_visible(55));
    }

    #[test]
    fn test_scroll_to_top_bottom() {
        let sc = ScrollController::new();
        let mut vp = Viewport::new(20, 80, 100);

        sc.scroll_to_bottom(&mut vp);
        assert_eq!(vp.scroll_row(), 80); // 100 - 20

        sc.scroll_to_top(&mut vp);
        assert_eq!(vp.scroll_row(), 0);
    }

    #[test]
    fn test_center_on_row() {
        let sc = ScrollController::new();
        let mut vp = Viewport::new(20, 80, 100);

        sc.center_on_row(&mut vp, 50);
        assert!(vp.is_line_visible(50));
    }

    #[test]
    fn test_set_scroll_speed() {
        let mut sc = ScrollController::new();
        sc.set_scroll_speed(5);
        assert_eq!(sc.scroll_speed(), 5);

        sc.set_scroll_speed(0); // Should clamp to 1
        assert_eq!(sc.scroll_speed(), 1);
    }

    #[test]
    fn test_horizontal_scroll() {
        let sc = ScrollController::new();
        let mut vp = Viewport::new(20, 80, 100);

        sc.scroll_right(&mut vp);
        assert_eq!(vp.scroll_col(), 4);

        sc.scroll_left(&mut vp);
        assert_eq!(vp.scroll_col(), 0);
    }
}
