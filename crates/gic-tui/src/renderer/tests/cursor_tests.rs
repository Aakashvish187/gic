//! Cursor rendering tests.

use crate::renderer::cursor_renderer::{cursor_shape_for_mode, CursorRenderer};
use crate::renderer::types::CursorShape;
use crate::renderer::viewport::Viewport;
use gic_core::CursorPosition;
use ratatui::layout::Rect;

#[test]
fn test_cursor_at_origin() {
    let cr = CursorRenderer::with_defaults();
    let viewport = Viewport::new(24, 80, 100);
    let text_area = Rect::new(5, 0, 75, 23);

    let info = cr.compute(CursorPosition::zero(), &viewport, text_area, Some("hello"));
    assert!(info.visible);
    assert_eq!(info.screen_position.col, 5); // text_area.x
    assert_eq!(info.screen_position.row, 0); // text_area.y
}

#[test]
fn test_cursor_at_end_of_line() {
    let cr = CursorRenderer::with_defaults();
    let viewport = Viewport::new(24, 80, 100);
    let text_area = Rect::new(5, 0, 75, 23);

    let info = cr.compute(
        CursorPosition::new(0, 5),
        &viewport,
        text_area,
        Some("Hello"),
    );
    assert!(info.visible);
    assert_eq!(info.screen_position.col, 10); // 5 (area.x) + 5 (col)
}

#[test]
fn test_cursor_scrolled_viewport() {
    let cr = CursorRenderer::with_defaults();
    let mut viewport = Viewport::new(10, 80, 100);
    viewport.scroll_down(20);
    let text_area = Rect::new(5, 0, 75, 9);

    // Cursor at row 25 → viewport row 5
    let info = cr.compute(
        CursorPosition::new(25, 0),
        &viewport,
        text_area,
        Some("hello"),
    );
    assert!(info.visible);
    assert_eq!(info.screen_position.row, 5);
}

#[test]
fn test_cursor_outside_viewport_invisible() {
    let cr = CursorRenderer::with_defaults();
    let viewport = Viewport::new(10, 80, 100);
    let text_area = Rect::new(5, 0, 75, 9);

    let info = cr.compute(CursorPosition::new(50, 0), &viewport, text_area, Some(""));
    assert!(!info.visible);
}

#[test]
fn test_cursor_with_wide_chars() {
    let cr = CursorRenderer::with_defaults();
    let viewport = Viewport::new(24, 80, 100);
    let text_area = Rect::new(4, 0, 76, 23);

    // "中文" is 4 display columns, cursor after it at char index 2
    let info = cr.compute(
        CursorPosition::new(0, 2),
        &viewport,
        text_area,
        Some("中文abc"),
    );
    assert!(info.visible);
    assert_eq!(info.screen_position.col, 8); // 4 (area.x) + 4 (display cols)
}

#[test]
fn test_all_cursor_shapes() {
    let shapes = [
        CursorShape::Block,
        CursorShape::Beam,
        CursorShape::Underline,
    ];
    for shape in &shapes {
        let cr = CursorRenderer::new(*shape, 4);
        assert_eq!(cr.shape(), *shape);
    }
}

#[test]
fn test_cursor_shape_for_all_modes() {
    let test_cases = [
        ("NORMAL", CursorShape::Block),
        ("INSERT", CursorShape::Beam),
        ("REPLACE", CursorShape::Underline),
        ("VISUAL", CursorShape::Block),
        ("COMMAND", CursorShape::Block),
    ];

    for (mode, expected) in &test_cases {
        assert_eq!(
            cursor_shape_for_mode(mode),
            *expected,
            "Mode '{}' returned wrong cursor shape",
            mode
        );
    }
}
