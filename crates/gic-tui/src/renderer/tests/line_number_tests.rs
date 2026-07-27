//! Line number rendering tests.

use crate::renderer::line_numbers::LineNumberRenderer;
use crate::renderer::themes::builtin;
use crate::renderer::types::LineNumberMode;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

#[test]
fn test_absolute_line_numbers_content() {
    let renderer = LineNumberRenderer::new(LineNumberMode::Absolute);
    let theme = builtin::gic_dark();
    let area = Rect::new(0, 0, 5, 5);
    let mut buf = Buffer::empty(area);

    renderer.render(&mut buf, area, 0, 20, 2, &theme);

    let content: String = buf
        .content()
        .iter()
        .map(|c| c.symbol().to_string())
        .collect();
    assert!(content.contains('1'));
    assert!(content.contains('2'));
    assert!(content.contains('3'));
    assert!(content.contains('4'));
    assert!(content.contains('5'));
}

#[test]
fn test_relative_line_numbers() {
    let renderer = LineNumberRenderer::new(LineNumberMode::Relative);
    let theme = builtin::gic_dark();
    let area = Rect::new(0, 0, 5, 5);
    let mut buf = Buffer::empty(area);

    // Current line at row 2 (3rd line)
    renderer.render(&mut buf, area, 0, 20, 2, &theme);

    let content: String = buf
        .content()
        .iter()
        .map(|c| c.symbol().to_string())
        .collect();
    // Row 2 should show absolute "3", others show relative distances
    assert!(content.contains('3')); // absolute on current line
    assert!(content.contains('1')); // relative distance
    assert!(content.contains('2')); // relative distance
}

#[test]
fn test_line_numbers_with_scroll_offset() {
    let renderer = LineNumberRenderer::new(LineNumberMode::Absolute);
    let theme = builtin::gic_dark();
    let area = Rect::new(0, 0, 6, 3);
    let mut buf = Buffer::empty(area);

    // Scrolled to row 98, showing lines 99-101
    renderer.render(&mut buf, area, 98, 200, 99, &theme);

    let content: String = buf
        .content()
        .iter()
        .map(|c| c.symbol().to_string())
        .collect();
    assert!(content.contains("99"));
    assert!(content.contains("100"));
}

#[test]
fn test_line_numbers_past_buffer_end() {
    let renderer = LineNumberRenderer::new(LineNumberMode::Absolute);
    let theme = builtin::gic_dark();
    let area = Rect::new(0, 0, 5, 10);
    let mut buf = Buffer::empty(area);

    // Only 3 lines but 10 rows
    renderer.render(&mut buf, area, 0, 3, 0, &theme);

    // Should not crash, extra rows should be empty
}

#[test]
fn test_line_number_large_file() {
    let renderer = LineNumberRenderer::new(LineNumberMode::Absolute);
    let theme = builtin::gic_dark();
    let area = Rect::new(0, 0, 8, 3);
    let mut buf = Buffer::empty(area);

    renderer.render(&mut buf, area, 99_997, 100_000, 99_998, &theme);

    let content: String = buf
        .content()
        .iter()
        .map(|c| c.symbol().to_string())
        .collect();
    assert!(content.contains("99999"));
}
