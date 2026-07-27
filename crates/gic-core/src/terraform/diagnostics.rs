//! Diagnostics Bridge for Terraform Engine.
//!
//! Converts internal `TerraformDiagnostic` items to central GIC `Diagnostic` objects.

use crate::diagnostics::diagnostic::Diagnostic;
use crate::diagnostics::position::DiagnosticPosition;
use crate::diagnostics::quick_fix::QuickFix;
use crate::diagnostics::range::DiagnosticRange;
use crate::diagnostics::severity::DiagnosticLevel;
use crate::parser::language::LanguageId;
use crate::terraform::validator::{TerraformDiagnostic, TerraformSeverity};

/// Converts a collection of `TerraformDiagnostic` items to GIC `Diagnostic` objects.
pub fn convert_terraform_diagnostics(
    source_diags: &[TerraformDiagnostic],
    language_id: LanguageId,
) -> Vec<Diagnostic> {
    source_diags
        .iter()
        .map(|d| convert_terraform_diagnostic(d, language_id.clone()))
        .collect()
}

/// Converts a single `TerraformDiagnostic` item to a GIC `Diagnostic`.
pub fn convert_terraform_diagnostic(
    diag: &TerraformDiagnostic,
    language_id: LanguageId,
) -> Diagnostic {
    let level = match diag.severity {
        TerraformSeverity::Error => DiagnosticLevel::Error,
        TerraformSeverity::Warning => DiagnosticLevel::Warning,
        TerraformSeverity::Info => DiagnosticLevel::Information,
        TerraformSeverity::Hint => DiagnosticLevel::Hint,
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
