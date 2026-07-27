//! YAML Diagnostics Bridge.
//!
//! Converts internal `YamlValidationDiagnostic` items into `gic_core::diagnostics::Diagnostic` objects
//! compatible with the central GIC Diagnostics and UI subsystem.

use crate::diagnostics::diagnostic::Diagnostic;
use crate::diagnostics::position::DiagnosticPosition;
use crate::diagnostics::quick_fix::QuickFix;
use crate::diagnostics::range::DiagnosticRange;
use crate::diagnostics::severity::DiagnosticLevel;
use crate::parser::language::LanguageId;
use crate::yaml::validator::{YamlSeverity, YamlValidationDiagnostic};

/// Converts a internal `YamlValidationDiagnostic` into a system `Diagnostic`.
pub fn convert_yaml_diagnostic(diag: YamlValidationDiagnostic) -> Diagnostic {
    let severity = match diag.severity {
        YamlSeverity::Error => DiagnosticLevel::Error,
        YamlSeverity::Warning => DiagnosticLevel::Warning,
        YamlSeverity::Info => DiagnosticLevel::Information,
        YamlSeverity::Hint => DiagnosticLevel::Hint,
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

    if let Some(qf) = diag.quick_fix {
        let quick_fix = QuickFix::replacement(qf.title, range, qf.replacement);
        core_diag.add_quick_fix(quick_fix);
    }

    core_diag
}

/// Converts a collection of YAML diagnostics into central `Diagnostic` items.
pub fn convert_yaml_diagnostics(diagnostics: Vec<YamlValidationDiagnostic>) -> Vec<Diagnostic> {
    diagnostics
        .into_iter()
        .map(convert_yaml_diagnostic)
        .collect()
}
