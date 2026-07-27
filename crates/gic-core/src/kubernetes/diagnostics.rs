//! Kubernetes Diagnostics Bridge.
//!
//! Converts internal `K8sDiagnostic` objects into central `gic_core::diagnostics::Diagnostic` instances
//! for UI rendering and system diagnostic notifications.

use crate::diagnostics::diagnostic::Diagnostic;
use crate::diagnostics::position::DiagnosticPosition;
use crate::diagnostics::quick_fix::QuickFix;
use crate::diagnostics::range::DiagnosticRange;
use crate::diagnostics::severity::DiagnosticLevel;
use crate::kubernetes::validator::{K8sDiagnostic, K8sSeverity};
use crate::parser::language::LanguageId;

/// Converts a internal `K8sDiagnostic` into a system `Diagnostic`.
pub fn convert_k8s_diagnostic(diag: K8sDiagnostic) -> Diagnostic {
    let severity = match diag.severity {
        K8sSeverity::Error => DiagnosticLevel::Error,
        K8sSeverity::Warning => DiagnosticLevel::Warning,
        K8sSeverity::Info => DiagnosticLevel::Information,
        K8sSeverity::Hint => DiagnosticLevel::Hint,
    };

    let start_pos = DiagnosticPosition::new(
        diag.span.start.line,
        diag.span.start.column,
        diag.span.start.byte_offset,
    );
    let end_pos = DiagnosticPosition::new(
        diag.span.end.line,
        diag.span.end.column,
        diag.span.end.byte_offset,
    );
    let range = DiagnosticRange::new(start_pos, end_pos);

    let mut core_diag = Diagnostic::new(
        severity,
        diag.message,
        range,
        diag.rule_id,
        LanguageId::Yaml,
    );

    if let Some((title, replacement)) = diag.quick_fix {
        let quick_fix = QuickFix::replacement(title, range, replacement);
        core_diag.add_quick_fix(quick_fix);
    }

    core_diag
}

/// Converts a vector of `K8sDiagnostic` items into system `Diagnostic` instances.
pub fn convert_k8s_diagnostics(diagnostics: Vec<K8sDiagnostic>) -> Vec<Diagnostic> {
    diagnostics
        .into_iter()
        .map(convert_k8s_diagnostic)
        .collect()
}
