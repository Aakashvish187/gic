//! Diagnostics Bridge for Docker Engine.
//!
//! Converts internal `DockerDiagnostic` items to central GIC `Diagnostic` objects.

use crate::diagnostics::diagnostic::Diagnostic;
use crate::diagnostics::position::DiagnosticPosition;
use crate::diagnostics::quick_fix::QuickFix;
use crate::diagnostics::range::DiagnosticRange;
use crate::diagnostics::severity::DiagnosticLevel;
use crate::docker::validator::{DockerDiagnostic, DockerSeverity};
use crate::parser::language::LanguageId;

/// Converts a collection of `DockerDiagnostic` items to GIC `Diagnostic` objects.
pub fn convert_docker_diagnostics(
    source_diags: &[DockerDiagnostic],
    language_id: LanguageId,
) -> Vec<Diagnostic> {
    source_diags
        .iter()
        .map(|d| convert_docker_diagnostic(d, language_id.clone()))
        .collect()
}

/// Converts a single `DockerDiagnostic` item to a GIC `Diagnostic`.
pub fn convert_docker_diagnostic(diag: &DockerDiagnostic, language_id: LanguageId) -> Diagnostic {
    let level = match diag.severity {
        DockerSeverity::Error => DiagnosticLevel::Error,
        DockerSeverity::Warning => DiagnosticLevel::Warning,
        DockerSeverity::Info => DiagnosticLevel::Information,
        DockerSeverity::Hint => DiagnosticLevel::Hint,
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

    let mut gic_diag = Diagnostic::new(
        level,
        diag.message.clone(),
        range,
        diag.rule_id.clone(),
        language_id,
    );

    if let Some((ref title, ref replacement)) = diag.quick_fix {
        let qf = QuickFix::replacement(title.clone(), range, replacement.clone());
        gic_diag = gic_diag.with_quick_fixes(vec![qf]);
    }

    gic_diag
}
