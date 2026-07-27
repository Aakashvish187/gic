//! Integration tests for the full render pipeline.

use crate::renderer::pipeline::RenderPipeline;
use crate::renderer::render_state::RenderState;
use crate::renderer::themes::builtin;
use crate::renderer::viewport::Viewport;
use gic_core::{CursorPosition, Document, EngineState, TextBuffer};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

fn make_pipeline_state<'a>(
    buffer: &'a TextBuffer,
    document: &'a Document,
    engine: &'a EngineState,
    viewport: &'a Viewport,
    theme: &'a crate::renderer::themes::Theme,
    cursor: CursorPosition,
) -> RenderState<'a> {
    RenderState::new(buffer, document, engine, viewport, theme, cursor)
}

#[test]
fn test_full_pipeline_empty_file() {
    let pipeline = RenderPipeline::new();
    let buffer = TextBuffer::new();
    let doc = Document::new_empty();
    let engine = EngineState::new();
    let viewport = Viewport::new(23, 80, buffer.line_count());
    let theme = builtin::gic_dark();

    let state = make_pipeline_state(
        &buffer,
        &doc,
        &engine,
        &viewport,
        &theme,
        CursorPosition::zero(),
    );

    let area = Rect::new(0, 0, 80, 24);
    let mut buf = Buffer::empty(area);

    let result = pipeline.render(&mut buf, area, &state);
    assert!(result.is_ok());

    let cursor_info = result.unwrap();
    assert!(cursor_info.visible);
}

#[test]
fn test_full_pipeline_rust_file() {
    let pipeline = RenderPipeline::new();
    let content = r#"use std::io;

fn main() {
    let name = "World";
    let count = 42;
    // Print greeting
    println!("Hello, {}! Count: {}", name, count);
}
"#;
    let buffer = TextBuffer::from_str(content);
    let mut doc = Document::new_empty();
    doc.set_path("main.rs");
    let engine = EngineState::new();
    let viewport = Viewport::new(23, 120, buffer.line_count());
    let theme = builtin::gic_dark();

    let state = make_pipeline_state(
        &buffer,
        &doc,
        &engine,
        &viewport,
        &theme,
        CursorPosition::new(3, 4),
    )
    .with_language("Rust");

    let area = Rect::new(0, 0, 120, 24);
    let mut buf = Buffer::empty(area);

    let result = pipeline.render(&mut buf, area, &state);
    assert!(result.is_ok());

    let content: String = buf
        .content()
        .iter()
        .map(|c| c.symbol().to_string())
        .collect();
    assert!(content.contains("main"));
    assert!(content.contains("println"));
}

#[test]
fn test_full_pipeline_yaml_file() {
    let pipeline = RenderPipeline::new();
    let content = "apiVersion: v1\nkind: Service\nmetadata:\n  name: my-service\n  namespace: default\nspec:\n  ports:\n    - port: 80\n";
    let buffer = TextBuffer::from_str(content);
    let mut doc = Document::new_empty();
    doc.set_path("service.yaml");
    let engine = EngineState::new();
    let viewport = Viewport::new(23, 80, buffer.line_count());
    let theme = builtin::gic_dark();

    let state = make_pipeline_state(
        &buffer,
        &doc,
        &engine,
        &viewport,
        &theme,
        CursorPosition::zero(),
    );

    let area = Rect::new(0, 0, 80, 24);
    let mut buf = Buffer::empty(area);

    let result = pipeline.render(&mut buf, area, &state);
    assert!(result.is_ok());
}

#[test]
fn test_full_pipeline_10k_lines() {
    let pipeline = RenderPipeline::new();
    let lines: Vec<String> = (0..10_000)
        .map(|i| format!("server.config.line_{} = \"value_{}\";", i, i))
        .collect();
    let buffer = TextBuffer::from_lines(lines);
    let doc = Document::new_empty();
    let engine = EngineState::new();
    let viewport = Viewport::new(23, 120, buffer.line_count());
    let theme = builtin::gic_dark();

    let state = make_pipeline_state(
        &buffer,
        &doc,
        &engine,
        &viewport,
        &theme,
        CursorPosition::new(5000, 10),
    );

    let area = Rect::new(0, 0, 120, 24);
    let mut buf = Buffer::empty(area);

    let result = pipeline.render(&mut buf, area, &state);
    assert!(result.is_ok());
}

#[test]
fn test_full_pipeline_tiny_terminal() {
    let pipeline = RenderPipeline::new();
    let buffer = TextBuffer::from_str("Hello World");
    let doc = Document::new_empty();
    let engine = EngineState::new();
    let viewport = Viewport::new(1, 10, buffer.line_count());
    let theme = builtin::gic_dark();

    let state = make_pipeline_state(
        &buffer,
        &doc,
        &engine,
        &viewport,
        &theme,
        CursorPosition::zero(),
    );

    let area = Rect::new(0, 0, 10, 2);
    let mut buf = Buffer::empty(area);

    let result = pipeline.render(&mut buf, area, &state);
    assert!(result.is_ok());
}

#[test]
fn test_full_pipeline_with_light_theme() {
    let pipeline = RenderPipeline::new();
    let buffer = TextBuffer::from_str("let x = 42;");
    let doc = Document::new_empty();
    let engine = EngineState::new();
    let viewport = Viewport::new(23, 80, buffer.line_count());
    let theme = builtin::gic_light();

    let state = make_pipeline_state(
        &buffer,
        &doc,
        &engine,
        &viewport,
        &theme,
        CursorPosition::zero(),
    );

    let area = Rect::new(0, 0, 80, 24);
    let mut buf = Buffer::empty(area);

    let result = pipeline.render(&mut buf, area, &state);
    assert!(result.is_ok());
}

#[test]
fn test_full_pipeline_with_high_contrast_theme() {
    let pipeline = RenderPipeline::new();
    let buffer = TextBuffer::from_str("fn main() {}");
    let doc = Document::new_empty();
    let engine = EngineState::new();
    let viewport = Viewport::new(23, 80, buffer.line_count());
    let theme = builtin::high_contrast();

    let state = make_pipeline_state(
        &buffer,
        &doc,
        &engine,
        &viewport,
        &theme,
        CursorPosition::zero(),
    );

    let area = Rect::new(0, 0, 80, 24);
    let mut buf = Buffer::empty(area);

    let result = pipeline.render(&mut buf, area, &state);
    assert!(result.is_ok());
}

#[test]
fn test_dirty_indicator_accuracy() {
    let pipeline = RenderPipeline::new();
    let mut buffer = TextBuffer::from_str("original");
    buffer.insert_str(" modified").unwrap();

    let doc = Document::new_empty();
    let engine = EngineState::new();
    let viewport = Viewport::new(23, 80, buffer.line_count());
    let theme = builtin::gic_dark();

    let state = make_pipeline_state(
        &buffer,
        &doc,
        &engine,
        &viewport,
        &theme,
        CursorPosition::zero(),
    );

    // Verify the state reports modified correctly
    assert!(state.is_modified());

    let area = Rect::new(0, 0, 80, 24);
    let mut buf = Buffer::empty(area);

    let result = pipeline.render(&mut buf, area, &state);
    assert!(result.is_ok());

    let content: String = buf
        .content()
        .iter()
        .map(|c| c.symbol().to_string())
        .collect();
    assert!(content.contains("[+]")); // Modified indicator in status bar
}
