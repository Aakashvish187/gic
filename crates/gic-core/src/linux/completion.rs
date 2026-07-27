//! Context-aware Shell Completion Contracts.

use crate::yaml::parser::Position;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CompletionKind {
    #[default]
    Command,
    OptionFlag,
    Filesystem,
    User,
    Group,
    Package,
    Variable,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LinuxCompletionItem {
    pub label: String,
    pub detail: String,
    pub kind: CompletionKind,
}

pub trait LinuxCompleter: Send + Sync {
    fn complete(&self, source: &str, position: Position) -> Vec<LinuxCompletionItem>;
}
