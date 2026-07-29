//! # Generic / Fallback Language Engine
//!
//! Returns no diagnostics, completions, or hover information.
//! Used for file types without a dedicated engine.

use super::{Completion, EngineDiagnostic, HoverInfo, LanguageEngine};

pub struct GenericEngine;

impl LanguageEngine for GenericEngine {
    fn name(&self) -> &'static str { "Plain Text" }
    fn id(&self) -> &'static str { "generic" }

    fn diagnostics(&self, _content: &str) -> Vec<EngineDiagnostic> {
        Vec::new()
    }

    fn completions(&self, _content: &str, _row: usize, _col: usize) -> Vec<Completion> {
        Vec::new()
    }

    fn hover(&self, _content: &str, _row: usize, _col: usize) -> Option<HoverInfo> {
        None
    }
}
