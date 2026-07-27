//! Layout engine tests.

use crate::renderer::layout::LayoutEngine;
use ratatui::layout::Rect;

#[test]
fn test_layout_80x24() {
    let area = Rect::new(0, 0, 80, 24);
    let layout = LayoutEngine::compute(area, 100);

    assert_eq!(layout.status_bar_area.height, 1);
    assert_eq!(layout.text_area.height, 23);
    assert!(layout.gutter_width >= 4);
    assert!(layout.has_text_area());
}

#[test]
fn test_layout_regions_cover_full_area() {
    let area = Rect::new(0, 0, 100, 30);
    let layout = LayoutEngine::compute(area, 500);

    // Gutter + text = full width
    assert_eq!(
        layout.line_number_area.width + layout.text_area.width,
        area.width
    );

    // Content + status = full height
    assert_eq!(
        layout.text_area.height + layout.status_bar_area.height,
        area.height
    );
}

#[test]
fn test_layout_single_line_file() {
    let area = Rect::new(0, 0, 80, 24);
    let layout = LayoutEngine::compute(area, 1);
    assert_eq!(layout.gutter_width, 4); // Minimum gutter
}

#[test]
fn test_layout_million_line_file() {
    let area = Rect::new(0, 0, 200, 50);
    let layout = LayoutEngine::compute(area, 1_000_000);
    // 7 digits + 2 padding = 9, but capped at width/4 = 50
    assert!(layout.gutter_width <= 50);
    assert!(layout.gutter_width >= 9);
}

#[test]
fn test_gutter_width_progression() {
    let widths: Vec<u16> = [1, 10, 100, 1000, 10_000, 100_000]
        .iter()
        .map(|n| LayoutEngine::calculate_gutter_width(*n))
        .collect();

    // Each step should be >= previous
    for i in 1..widths.len() {
        assert!(
            widths[i] >= widths[i - 1],
            "Gutter width should be non-decreasing: {} < {} at index {}",
            widths[i],
            widths[i - 1],
            i
        );
    }
}

#[test]
fn test_layout_offset_origin() {
    // Test with non-zero origin
    let area = Rect::new(5, 3, 80, 24);
    let layout = LayoutEngine::compute(area, 100);

    assert_eq!(layout.line_number_area.x, 5);
    assert_eq!(layout.line_number_area.y, 3);
    assert_eq!(layout.status_bar_area.x, 5);
}
