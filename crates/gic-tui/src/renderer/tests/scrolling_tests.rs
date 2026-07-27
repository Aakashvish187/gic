//! Scrolling controller tests.

use crate::renderer::scrolling::ScrollController;
use crate::renderer::viewport::Viewport;

#[test]
fn test_scroll_controller_follow_cursor_beyond_viewport() {
    let sc = ScrollController::with_scroll_off(3);
    let mut vp = Viewport::new(20, 80, 200);

    // Move cursor far below viewport
    sc.follow_cursor(&mut vp, 100, 0);
    assert!(vp.is_line_visible(100));
}

#[test]
fn test_scroll_controller_follow_cursor_with_large_scroll_off() {
    let sc = ScrollController::with_scroll_off(100); // Larger than viewport
    let mut vp = Viewport::new(20, 80, 200);

    // scroll_off should be clamped to viewport/2
    sc.follow_cursor(&mut vp, 50, 0);
    assert!(vp.is_line_visible(50));
}

#[test]
fn test_scroll_multiple_rapid_scrolls() {
    let sc = ScrollController::new();
    let mut vp = Viewport::new(20, 80, 1000);

    for _ in 0..100 {
        sc.scroll_down(&mut vp);
    }

    // Should not exceed buffer bounds
    assert!(vp.scroll_row() <= 980); // 1000 - 20
}

#[test]
fn test_resize_preserves_cursor_visibility() {
    let sc = ScrollController::new();
    let mut vp = Viewport::new(40, 120, 200);
    vp.scroll_down(100);

    // Resize to smaller terminal
    sc.handle_resize(&mut vp, 10, 60, 105, 0);

    assert!(vp.is_line_visible(105));
    assert_eq!(vp.visible_rows(), 10);
    assert_eq!(vp.visible_cols(), 60);
}

#[test]
fn test_horizontal_scroll_both_directions() {
    let sc = ScrollController::new();
    let mut vp = Viewport::new(20, 40, 100);

    sc.scroll_right(&mut vp);
    sc.scroll_right(&mut vp);
    assert!(vp.scroll_col() > 0);

    let saved = vp.scroll_col();
    sc.scroll_left(&mut vp);
    assert!(vp.scroll_col() < saved);
}

#[test]
fn test_scroll_to_top_and_bottom() {
    let sc = ScrollController::new();
    let mut vp = Viewport::new(20, 80, 500);

    sc.scroll_to_bottom(&mut vp);
    assert_eq!(vp.scroll_row(), 480);

    sc.scroll_to_top(&mut vp);
    assert_eq!(vp.scroll_row(), 0);
    assert_eq!(vp.scroll_col(), 0);
}
