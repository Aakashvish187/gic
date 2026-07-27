//! Status bar rendering tests.

use crate::renderer::file_info::FileInfo;
use crate::renderer::status_bar::StatusBarRenderer;
use crate::renderer::themes::builtin;
use gic_core::{Document, TextBuffer};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::Widget;

fn make_file_info(name: Option<&str>, modified: bool) -> FileInfo {
    let mut doc = Document::new_empty();
    if let Some(n) = name {
        doc.set_path(n);
    }
    let mut buffer = TextBuffer::from_str("Hello\nWorld\nTest");
    if modified {
        buffer.insert_str("x").unwrap();
    }
    FileInfo::from_state(&doc, &buffer, 1, 3, Some("Rust"))
}

#[test]
fn test_status_bar_contains_mode() {
    let info = make_file_info(None, false);
    let theme = builtin::gic_dark();
    let bar = StatusBarRenderer::new(&info, "NORMAL", &theme, (80, 24));

    let area = Rect::new(0, 0, 80, 1);
    let mut buf = Buffer::empty(area);
    bar.render(area, &mut buf);

    let content: String = buf
        .content()
        .iter()
        .map(|c| c.symbol().to_string())
        .collect();
    assert!(content.contains("NORMAL"));
}

#[test]
fn test_status_bar_contains_file_name() {
    let info = make_file_info(Some("z:/project/config.yaml"), false);
    let theme = builtin::gic_dark();
    let bar = StatusBarRenderer::new(&info, "NORMAL", &theme, (120, 40));

    let area = Rect::new(0, 0, 120, 1);
    let mut buf = Buffer::empty(area);
    bar.render(area, &mut buf);

    let content: String = buf
        .content()
        .iter()
        .map(|c| c.symbol().to_string())
        .collect();
    assert!(content.contains("config.yaml"));
}

#[test]
fn test_status_bar_shows_modified_indicator() {
    let info = make_file_info(None, true);
    let theme = builtin::gic_dark();
    let bar = StatusBarRenderer::new(&info, "INSERT", &theme, (80, 24));

    let area = Rect::new(0, 0, 80, 1);
    let mut buf = Buffer::empty(area);
    bar.render(area, &mut buf);

    let content: String = buf
        .content()
        .iter()
        .map(|c| c.symbol().to_string())
        .collect();
    assert!(content.contains("[+]"));
}

#[test]
fn test_status_bar_shows_cursor_position() {
    let info = make_file_info(None, false);
    let theme = builtin::gic_dark();
    let bar = StatusBarRenderer::new(&info, "NORMAL", &theme, (120, 40));

    let area = Rect::new(0, 0, 120, 1);
    let mut buf = Buffer::empty(area);
    bar.render(area, &mut buf);

    let content: String = buf
        .content()
        .iter()
        .map(|c| c.symbol().to_string())
        .collect();
    // Cursor at (1, 3) → display as "Ln 2, Col 4"
    assert!(content.contains("Ln 2"));
}

#[test]
fn test_status_bar_very_narrow() {
    let info = make_file_info(None, false);
    let theme = builtin::gic_dark();
    let bar = StatusBarRenderer::new(&info, "N", &theme, (10, 5));

    let area = Rect::new(0, 0, 10, 1);
    let mut buf = Buffer::empty(area);
    // Should not panic
    bar.render(area, &mut buf);
}
