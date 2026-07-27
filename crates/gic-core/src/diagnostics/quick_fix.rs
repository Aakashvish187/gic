//! Quick fix architectural constructs for diagnostic resolution suggestions.

use crate::diagnostics::range::DiagnosticRange;
use serde::{Deserialize, Serialize};

/// Represents a individual text edit operation proposed by a QuickFix.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextEdit {
    /// The range in the document to be replaced or edited.
    pub range: DiagnosticRange,
    /// The new text content to substitute.
    pub new_text: String,
}

impl TextEdit {
    /// Creates a replacement text edit.
    pub fn replace(range: DiagnosticRange, new_text: impl Into<String>) -> Self {
        Self {
            range,
            new_text: new_text.into(),
        }
    }

    /// Creates an insertion text edit at a single point.
    pub fn insert(at: DiagnosticRange, text: impl Into<String>) -> Self {
        Self {
            range: at,
            new_text: text.into(),
        }
    }

    /// Creates a deletion text edit.
    pub fn delete(range: DiagnosticRange) -> Self {
        Self {
            range,
            new_text: String::new(),
        }
    }
}

/// Categorizes the type of quick fix resolution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum QuickFixKind {
    /// Suggested string replacement for a node or value.
    SuggestedReplacement { replacement: String },
    /// Text insertion at a specific position.
    InsertText { text: String },
    /// Deletion of target text or node.
    DeleteText,
    /// Direct replacement of a specified range.
    ReplaceRange { replacement: String },
    /// Placeholder for future complex AI/plugin auto-fix actions.
    FutureAutoFix {
        action_id: String,
        payload: Option<String>,
    },
}

/// Represents a single proposed quick fix suggestion attached to a diagnostic report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuickFix {
    /// Human-readable title describing the quick fix.
    pub title: String,
    /// The quick fix category/kind.
    pub kind: QuickFixKind,
    /// Primary range associated with this quick fix.
    pub range: DiagnosticRange,
    /// Concrete text edits required to perform this quick fix.
    pub edits: Vec<TextEdit>,
    /// Indicates whether this quick fix is the preferred default resolution.
    pub is_preferred: bool,
}

impl QuickFix {
    /// Creates a new quick fix.
    pub fn new(
        title: impl Into<String>,
        kind: QuickFixKind,
        range: DiagnosticRange,
        edits: Vec<TextEdit>,
    ) -> Self {
        Self {
            title: title.into(),
            kind,
            range,
            edits,
            is_preferred: false,
        }
    }

    /// Marks this quick fix as preferred.
    pub fn with_preferred(mut self, preferred: bool) -> Self {
        self.is_preferred = preferred;
        self
    }

    /// Creates a quick fix for a suggested string replacement.
    pub fn replacement(
        title: impl Into<String>,
        range: DiagnosticRange,
        replacement: impl Into<String>,
    ) -> Self {
        let rep_str = replacement.into();
        let edit = TextEdit::replace(range, rep_str.clone());
        Self::new(
            title,
            QuickFixKind::SuggestedReplacement {
                replacement: rep_str,
            },
            range,
            vec![edit],
        )
    }

    /// Creates a quick fix for text insertion.
    pub fn insert(
        title: impl Into<String>,
        range: DiagnosticRange,
        text: impl Into<String>,
    ) -> Self {
        let text_str = text.into();
        let edit = TextEdit::insert(range, text_str.clone());
        Self::new(
            title,
            QuickFixKind::InsertText { text: text_str },
            range,
            vec![edit],
        )
    }

    /// Creates a quick fix for text deletion.
    pub fn delete(title: impl Into<String>, range: DiagnosticRange) -> Self {
        let edit = TextEdit::delete(range);
        Self::new(title, QuickFixKind::DeleteText, range, vec![edit])
    }

    /// Creates a future auto-fix placeholder.
    pub fn future_autofix(
        title: impl Into<String>,
        range: DiagnosticRange,
        action_id: impl Into<String>,
    ) -> Self {
        let action = action_id.into();
        Self::new(
            title,
            QuickFixKind::FutureAutoFix {
                action_id: action,
                payload: None,
            },
            range,
            Vec::new(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostics::position::DiagnosticPosition;

    #[test]
    fn test_quick_fix_creation() {
        let p1 = DiagnosticPosition::new(1, 1, 0);
        let p2 = DiagnosticPosition::new(1, 5, 4);
        let range = DiagnosticRange::new(p1, p2);

        let qf = QuickFix::replacement("Fix indentation", range, "    ");
        assert_eq!(qf.title, "Fix indentation");
        assert_eq!(qf.edits.len(), 1);
        assert_eq!(qf.edits[0].new_text, "    ");
        assert!(matches!(qf.kind, QuickFixKind::SuggestedReplacement { .. }));
    }
}
