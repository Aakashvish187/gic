use crate::parser::TextRange;

/// Specific mutation operations that can be performed as part of a quick fix.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QuickFixOperation {
    /// Inserts text at a specific character position.
    Insert { position: usize, text: String },
    /// Replaces the text within the given range.
    Replace { range: TextRange, text: String },
    /// Deletes the text within the given range.
    Delete { range: TextRange },
    /// Renames a symbol.
    Rename { range: TextRange, new_name: String },
    /// Moves a block of text to a new position.
    Move {
        source: TextRange,
        target_position: usize,
    },
}

/// Represents an automated fix that can be applied to resolve a rule violation.
#[derive(Debug, Clone)]
pub struct RuleQuickFix {
    /// A short title explaining what the fix does (e.g., "Add missing attribute").
    pub title: String,
    /// The operations required to apply the fix.
    pub operations: Vec<QuickFixOperation>,
}

impl RuleQuickFix {
    /// Creates a new quick fix with the given title.
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            operations: Vec::new(),
        }
    }

    /// Adds an operation to the quick fix.
    pub fn with_operation(mut self, op: QuickFixOperation) -> Self {
        self.operations.push(op);
        self
    }
}
