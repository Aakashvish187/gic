//! GIC Diagnostics & Validation Engine module.
//!
//! The intelligence layer of GIC responsible for syntax tree validation,
//! rule evaluation, severity assignment, diagnostic creation, quick fix suggestions,
//! and high-performance result caching.

pub mod cache;
pub mod diagnostic;
pub mod engine;
pub mod errors;
pub mod formatter;
pub mod position;
pub mod quick_fix;
pub mod range;
pub mod registry;
pub mod rule;
pub mod severity;
pub mod utils;
pub mod validator;

#[cfg(test)]
pub mod tests;

pub use cache::{DiagnosticCache, DiagnosticCacheEntry};
pub use diagnostic::Diagnostic;
pub use engine::{ValidationEngine, ValidationMetrics};
pub use errors::{DiagnosticError, DiagnosticResult};
pub use formatter::{
    DiagnosticFormatter, JsonFormatter, PlainTextFormatter, PrettyTerminalFormatter,
};
pub use position::DiagnosticPosition;
pub use quick_fix::{QuickFix, QuickFixKind, TextEdit};
pub use range::DiagnosticRange;
pub use registry::{RuleLoader, RuleRegistry, ValidatorRegistry};
pub use rule::{Rule, RuleCategory, RuleConfig, RuleMetadata, RulePriority};
pub use severity::DiagnosticLevel;
pub use utils::{
    current_timestamp_ms, generate_diagnostic_id, offset_to_position, offsets_to_range,
};
pub use validator::{CoreSyntaxValidator, GenericRuleValidator, ValidationContext, Validator};
