//! Diagnostics Bridge for Linux Engine.
//!
//! Converts internal `LinuxDiagnostic` items to central GIC `Diagnostic` objects.

use crate::diagnostics::diagnostic::Diagnostic;
use crate::diagnostics::position::DiagnosticPosition;
use crate::diagnostics::range::DiagnosticRange;
use crate::diagnostics::severity::DiagnosticLevel;
use crate::linux::validator::LinuxDiagnostic;
use crate::parser::language::LanguageId;

pub fn convert_linux_diagnostics(
    source_diags: &[LinuxDiagnostic],
    language_id: LanguageId,
) -> Vec<Diagnostic> {
    source_diags
        .iter()
        .map(|d| convert_linux_diagnostic(d, language_id.clone()))
        .collect()
}

pub fn convert_linux_diagnostic(diag: &LinuxDiagnostic, language_id: LanguageId) -> Diagnostic {
    let level = if diag.is_error {
        DiagnosticLevel::Error
    } else {
        DiagnosticLevel::Warning
    };

    let start = DiagnosticPosition::new(
        diag.span.start.line,
        diag.span.start.column,
        diag.span.start.byte_offset,
    );
    let end = DiagnosticPosition::new(
        diag.span.end.line,
        diag.span.end.column,
        diag.span.end.byte_offset,
    );
    let range = DiagnosticRange::new(start, end);

    Diagnostic::new(
        level,
        diag.message.clone(),
        range,
        diag.rule_id.clone(),
        language_id,
    )
}
