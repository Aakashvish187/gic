use super::{configuration::WorkspaceSettings, rule::RuleContext};
use crate::parser::SyntaxNode;
use std::any::Any;

/// Provides the necessary context to a rule during evaluation.
pub struct EvaluationContext<'a> {
    /// The path of the file being evaluated.
    pub file_path: &'a str,
    /// The root AST node for the current evaluation context.
    pub root_node: Option<&'a SyntaxNode>,
    /// The workspace settings, which may contain rule-specific overrides.
    pub settings: &'a WorkspaceSettings,
    /// The programming language or DSL being evaluated.
    pub language: &'a str,
}

impl<'a> RuleContext for EvaluationContext<'a> {
    fn as_any(&self) -> &dyn Any {
        // Rust does not easily allow casting a struct with lifetimes to Any without 'static.
        // For the purpose of the architecture, we provide a placeholder or return a unit if needed.
        // We'll leave it unimplemented or provide a stub for now.
        unimplemented!(
            "Casting EvaluationContext to Any is not fully supported without static bounds."
        )
    }
}
