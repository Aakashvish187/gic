//! Quick fix structural and resolution proposal tests.

use crate::diagnostics::position::DiagnosticPosition;
use crate::diagnostics::quick_fix::{QuickFix, QuickFixKind};
use crate::diagnostics::range::DiagnosticRange;

#[test]
fn test_quick_fix_kinds_and_edits() {
    let p1 = DiagnosticPosition::new(1, 1, 0);
    let p2 = DiagnosticPosition::new(1, 4, 3);
    let range = DiagnosticRange::new(p1, p2);

    let qf_replace =
        QuickFix::replacement("Convert tabs to spaces", range, "  ").with_preferred(true);
    assert!(qf_replace.is_preferred);
    assert_eq!(qf_replace.edits.len(), 1);
    assert_eq!(qf_replace.edits[0].new_text, "  ");

    let qf_insert = QuickFix::insert("Add missing key", range, "version: '3'");
    assert!(matches!(qf_insert.kind, QuickFixKind::InsertText { .. }));

    let qf_delete = QuickFix::delete("Remove deprecated field", range);
    assert!(matches!(qf_delete.kind, QuickFixKind::DeleteText));
    assert_eq!(qf_delete.edits[0].new_text, "");

    let qf_future = QuickFix::future_autofix("Auto-format with AI", range, "ai_format_action");
    assert!(matches!(qf_future.kind, QuickFixKind::FutureAutoFix { .. }));
}
