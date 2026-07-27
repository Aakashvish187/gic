//! Hover Documentation Interfaces.

use crate::yaml::parser::Position;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct HoverDoc {
    pub markdown_content: String,
}

pub trait LinuxHoverProvider: Send + Sync {
    fn hover(&self, source: &str, position: Position) -> Option<HoverDoc>;
}
