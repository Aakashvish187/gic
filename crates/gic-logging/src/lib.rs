//! # gic-logging
//! Structured logging infrastructure for GIC using tracing and file output.

pub mod subscriber;

pub use subscriber::init_logging;
