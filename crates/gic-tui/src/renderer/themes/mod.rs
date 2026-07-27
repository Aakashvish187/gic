//! # Theme System
//!
//! Provides a theme abstraction, built-in themes, and a theme manager
//! for the GIC rendering engine. Themes define all colors used in the
//! editor UI, from syntax highlighting to the status bar.

pub mod builtin;
pub mod manager;
pub mod theme;

pub use builtin::{gic_dark, gic_light, high_contrast};
pub use manager::ThemeManager;
pub use theme::{SyntaxColors, Theme};
