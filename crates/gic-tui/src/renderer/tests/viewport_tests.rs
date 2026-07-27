//! Viewport engine tests.

use crate::renderer::viewport::Viewport;

#[test]
fn test_viewport_single_line_buffer() {
    let vp = Viewport::new(24, 80, 1);
    assert_eq!(vp.visible_line_range(), (0, 1));
    assert_eq!(vp.visible_line_count(), 1);
    assert!(vp.is_line_visible(0));
    assert!(!vp.is_line_visible(1));
}

#[test]
fn test_viewport_scroll_clamp_near_end() {
    let mut vp = Viewport::new(10, 80, 15);
    vp.scroll_down(10);
    // max scroll = 15 - 10 = 5
    assert_eq!(vp.scroll_row(), 5);
}

#[test]
fn test_viewport_ensure_cursor_visible_edge() {
    let mut vp = Viewport::new(10, 80, 100);
    // Cursor at exact bottom edge with scroll_off = 0
    vp.ensure_cursor_visible(9, 0, 0);
    assert_eq!(vp.scroll_row(), 0);
    assert!(vp.is_line_visible(9));
}

#[test]
fn test_viewport_page_at_boundary() {
    let mut vp = Viewport::new(10, 80, 15);
    vp.page_down();
    assert_eq!(vp.scroll_row(), 5); // clamped to max
    vp.page_down();
    assert_eq!(vp.scroll_row(), 5); // already at max
}

#[test]
fn test_viewport_resize_smaller() {
    let mut vp = Viewport::new(20, 80, 100);
    vp.scroll_down(85);
    assert_eq!(vp.scroll_row(), 80);

    vp.resize(5, 80);
    assert_eq!(vp.visible_rows(), 5);
    assert_eq!(vp.scroll_row(), 80); // Still valid: max = 100 - 5 = 95
}

#[test]
fn test_viewport_set_total_lines_to_one() {
    let mut vp = Viewport::new(10, 80, 100);
    vp.scroll_down(50);
    vp.set_total_lines(1);
    assert_eq!(vp.scroll_row(), 0); // Clamped: 1 - 10 = 0
}
