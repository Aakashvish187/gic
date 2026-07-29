//! # gic-tui
//! Interface adapter module providing Ratatui / Crossterm lifecycle management, event streams, and rendering engines.

pub mod event_stream;
pub mod render_engine;
pub mod renderer;
pub mod terminal;

pub use event_stream::EventStream;
pub use render_engine::RenderEngine;
pub use terminal::TerminalEngine;
