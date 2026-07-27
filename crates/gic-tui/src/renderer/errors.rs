//! # Rendering Engine Errors
//!
//! Defines the error types specific to the GIC rendering engine.
//! All rendering operations return `Result<T, RenderError>` to ensure
//! graceful degradation — the renderer must never panic.

use thiserror::Error;

/// Comprehensive error type for all rendering engine failures.
///
/// Each variant captures enough context for logging and recovery
/// without exposing internal implementation details to callers.
#[derive(Debug, Error)]
pub enum RenderError {
    /// Failed to compute editor layout from terminal dimensions.
    #[error("Layout calculation failed: {0}")]
    Layout(String),

    /// Viewport state is invalid or out-of-bounds.
    #[error("Viewport error: {0}")]
    Viewport(String),

    /// Theme loading or resolution failed.
    #[error("Theme error: {0}")]
    Theme(String),

    /// Syntax highlighting engine encountered an error.
    #[error("Syntax highlighting error: {0}")]
    Syntax(String),

    /// Terminal I/O or rendering backend failure.
    #[error("Terminal rendering error: {0}")]
    Terminal(String),

    /// A rendering operation received invalid dimensions (zero width/height).
    #[error("Invalid dimensions: width={width}, height={height}")]
    InvalidDimensions {
        /// Terminal or area width in columns.
        width: u16,
        /// Terminal or area height in rows.
        height: u16,
    },
}

impl RenderError {
    /// Creates a layout error from a descriptive message.
    pub fn layout<S: Into<String>>(msg: S) -> Self {
        Self::Layout(msg.into())
    }

    /// Creates a viewport error from a descriptive message.
    pub fn viewport<S: Into<String>>(msg: S) -> Self {
        Self::Viewport(msg.into())
    }

    /// Creates a theme error from a descriptive message.
    pub fn theme<S: Into<String>>(msg: S) -> Self {
        Self::Theme(msg.into())
    }

    /// Creates a syntax error from a descriptive message.
    pub fn syntax<S: Into<String>>(msg: S) -> Self {
        Self::Syntax(msg.into())
    }

    /// Creates a terminal error from a descriptive message.
    pub fn terminal<S: Into<String>>(msg: S) -> Self {
        Self::Terminal(msg.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_error_display() {
        let err = RenderError::layout("gutter too wide");
        assert_eq!(
            format!("{err}"),
            "Layout calculation failed: gutter too wide"
        );
    }

    #[test]
    fn test_render_error_invalid_dimensions() {
        let err = RenderError::InvalidDimensions {
            width: 0,
            height: 0,
        };
        assert_eq!(format!("{err}"), "Invalid dimensions: width=0, height=0");
    }

    #[test]
    fn test_render_error_factory_methods() {
        let e1 = RenderError::viewport("out of bounds");
        assert!(matches!(e1, RenderError::Viewport(_)));

        let e2 = RenderError::theme("unknown theme");
        assert!(matches!(e2, RenderError::Theme(_)));

        let e3 = RenderError::syntax("unterminated string");
        assert!(matches!(e3, RenderError::Syntax(_)));

        let e4 = RenderError::terminal("flush failed");
        assert!(matches!(e4, RenderError::Terminal(_)));
    }
}
