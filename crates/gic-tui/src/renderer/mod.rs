//! # GIC Rendering Engine
//!
//! The rendering engine is the display layer of GIC (General Infrastructure Console).
//! It is responsible exclusively for transforming immutable application state into
//! terminal UI output. It never modifies text buffers, processes commands, or owns
//! application data.
//!
//! ## Architecture
//!
//! The renderer follows a pipeline architecture:
//!
//! ```text
//! Application State (immutable refs)
//!         │
//!    LayoutEngine  →  splits terminal rect into regions
//!         │
//!      Viewport    →  calculates visible lines/columns
//!         │
//!    ┌────────────────────────────────────────┐
//!    │  Per-line rendering (single pass):     │
//!    │    Line Numbers + Syntax + Selection   │
//!    │    + Current Line Highlight            │
//!    └────────────────────────────────────────┘
//!         │
//!    CursorRenderer  →  terminal cursor placement
//!         │
//!    StatusBar        →  bottom status bar
//!         │
//!    Frame → Terminal
//! ```
//!
//! ## Design Principles
//!
//! - **Read-only**: Receives `&` references only, never `&mut`.
//! - **Zero allocations on hot path**: Pre-allocated buffers where possible.
//! - **Incremental rendering**: Only redraws what changed.
//! - **Modular**: Each concern (syntax, cursor, line numbers) is a separate module.
//! - **Trait-based extensibility**: New syntax backends, themes, and status bar
//!   widgets can be added without modifying existing code.
//!
//! ## Module Organization
//!
//! - [`errors`]: Error types for graceful failure recovery.
//! - [`types`]: Shared value types (coordinates, shapes, styled spans).
//! - [`viewport`]: Viewport calculations and scroll state.
//! - [`layout`]: Terminal area partitioning into editor regions.
//! - [`pipeline`]: The main `RenderPipeline` orchestrator.
//! - [`render_state`]: Immutable state bundle passed through the pipeline.
//! - [`text_renderer`]: Text line rendering with tab expansion and clipping.
//! - [`cursor_renderer`]: Cursor positioning and shape rendering.
//! - [`scrolling`]: Scroll controller with context margins.
//! - [`line_numbers`]: Line number gutter rendering.
//! - [`status_bar`]: Status bar with extensible widget system.
//! - [`file_info`]: File information extraction for display.
//! - [`dirty_indicator`]: Modified/saved state indicators.
//! - [`syntax`]: Syntax highlighting engine (trait + regex backend).
//! - [`themes`]: Theme system with built-in dark/light/high-contrast themes.

pub mod cursor_renderer;
pub mod dirty_indicator;
pub mod errors;
pub mod file_info;
pub mod layout;
pub mod line_numbers;
pub mod pipeline;
pub mod render_state;
pub mod scrolling;
pub mod status_bar;
pub mod syntax;
pub mod text_renderer;
pub mod themes;
pub mod types;
pub mod viewport;

#[cfg(test)]
mod tests;

// Re-export primary public API types for convenience.
pub use cursor_renderer::CursorRenderer;
pub use dirty_indicator::DirtyIndicator;
pub use errors::RenderError;
pub use file_info::FileInfo;
pub use layout::{EditorLayout, LayoutEngine};
pub use line_numbers::LineNumberRenderer;
pub use pipeline::RenderPipeline;
pub use render_state::RenderState;
pub use scrolling::ScrollController;
pub use status_bar::StatusBarRenderer;
pub use syntax::{HighlightedToken, SyntaxHighlighter, TokenKind};
pub use themes::{Theme, ThemeManager};
pub use types::{CursorShape, LineNumberMode, ScreenPosition, ScrollDirection, StyledSpan};
pub use viewport::Viewport;
