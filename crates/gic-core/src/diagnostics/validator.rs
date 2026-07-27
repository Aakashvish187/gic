//! Validator trait and execution context for language validation.

use crate::diagnostics::diagnostic::Diagnostic;
use crate::diagnostics::errors::{DiagnosticError, DiagnosticResult};
use crate::diagnostics::range::DiagnosticRange;
use crate::diagnostics::rule::Rule;
use crate::parser::language::LanguageId;
use crate::parser::tree::SyntaxTree;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Execution context passed to validators containing source text, rules, and flags.
pub struct ValidationContext<'a> {
    /// Full source text buffer reference.
    pub source_text: &'a str,
    /// Language format of document.
    pub language: LanguageId,
    /// Dirty/modified regions for incremental validation (empty means full validation).
    pub dirty_regions: Vec<DiagnosticRange>,
    /// Cancellation flag signal to halt long-running validation.
    pub cancel_flag: Arc<AtomicBool>,
}

impl<'a> ValidationContext<'a> {
    /// Creates a new full document `ValidationContext`.
    pub fn new(source_text: &'a str, language: LanguageId) -> Self {
        Self {
            source_text,
            language,
            dirty_regions: Vec::new(),
            cancel_flag: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Sets dirty regions for incremental validation.
    pub fn with_dirty_regions(mut self, dirty: Vec<DiagnosticRange>) -> Self {
        self.dirty_regions = dirty;
        self
    }

    /// Sets a custom cancellation flag signal.
    pub fn with_cancel_flag(mut self, flag: Arc<AtomicBool>) -> Self {
        self.cancel_flag = flag;
        self
    }

    /// Checks if validation has been requested to cancel.
    pub fn is_cancelled(&self) -> bool {
        self.cancel_flag.load(Ordering::Relaxed)
    }

    /// Throws error if validation was cancelled.
    pub fn check_cancelled(&self) -> DiagnosticResult<()> {
        if self.is_cancelled() {
            Err(DiagnosticError::ValidationCancelled)
        } else {
            Ok(())
        }
    }
}

/// Common trait that all language validators must implement.
pub trait Validator: Send + Sync {
    /// Returns the target language supported by this validator.
    fn language(&self) -> LanguageId;

    /// Returns human-readable name of the validator.
    fn name(&self) -> &str;

    /// Validates a syntax tree and returns generated diagnostics.
    fn validate(
        &self,
        tree: &SyntaxTree,
        ctx: &ValidationContext,
    ) -> DiagnosticResult<Vec<Diagnostic>>;
}

/// Generic rule-based validator that runs a set of rules against a syntax tree.
pub struct GenericRuleValidator {
    name: String,
    language: LanguageId,
    rules: Vec<Box<dyn Rule>>,
}

impl GenericRuleValidator {
    /// Creates a new generic rule validator.
    pub fn new(name: impl Into<String>, language: LanguageId, rules: Vec<Box<dyn Rule>>) -> Self {
        Self {
            name: name.into(),
            language,
            rules,
        }
    }
}

impl Validator for GenericRuleValidator {
    fn language(&self) -> LanguageId {
        self.language.clone()
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn validate(
        &self,
        tree: &SyntaxTree,
        ctx: &ValidationContext,
    ) -> DiagnosticResult<Vec<Diagnostic>> {
        let mut diagnostics = Vec::new();

        for rule in &self.rules {
            ctx.check_cancelled()?;

            if rule.supports_language(ctx.language.clone()) {
                let rule_diags = rule.evaluate(tree, ctx)?;
                diagnostics.extend(rule_diags);
            }
        }

        Ok(diagnostics)
    }
}

/// Core syntax validator checking built-in AST error nodes from the parser.
#[derive(Debug, Default, Clone, Copy)]
pub struct CoreSyntaxValidator;

impl Validator for CoreSyntaxValidator {
    fn language(&self) -> LanguageId {
        LanguageId::PlainText
    }

    fn name(&self) -> &str {
        "CoreSyntaxValidator"
    }

    fn validate(
        &self,
        tree: &SyntaxTree,
        ctx: &ValidationContext,
    ) -> DiagnosticResult<Vec<Diagnostic>> {
        ctx.check_cancelled()?;
        let mut diagnostics = Vec::new();

        // Convert raw parse diagnostics to full Diagnostic objects
        for p_diag in &tree.diagnostics {
            let diag_range = crate::diagnostics::range::DiagnosticRange::from(p_diag.range);
            let diag_level = match p_diag.severity {
                crate::parser::diagnostics::DiagnosticSeverity::Error => {
                    crate::diagnostics::severity::DiagnosticLevel::Error
                }
                crate::parser::diagnostics::DiagnosticSeverity::Warning => {
                    crate::diagnostics::severity::DiagnosticLevel::Warning
                }
                crate::parser::diagnostics::DiagnosticSeverity::Information => {
                    crate::diagnostics::severity::DiagnosticLevel::Information
                }
                crate::parser::diagnostics::DiagnosticSeverity::Hint => {
                    crate::diagnostics::severity::DiagnosticLevel::Hint
                }
            };

            let diagnostic = Diagnostic::new(
                diag_level,
                p_diag.message.clone(),
                diag_range,
                p_diag.source.clone(),
                tree.language.clone(),
            );

            diagnostics.push(diagnostic);
        }

        Ok(diagnostics)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_context_cancellation() {
        let text = "sample text";
        let ctx = ValidationContext::new(text, LanguageId::Yaml);
        assert!(!ctx.is_cancelled());
        assert!(ctx.check_cancelled().is_ok());

        let cancel_flag = Arc::new(AtomicBool::new(true));
        let ctx_cancelled =
            ValidationContext::new(text, LanguageId::Yaml).with_cancel_flag(cancel_flag);
        assert!(ctx_cancelled.is_cancelled());
        assert_eq!(
            ctx_cancelled.check_cancelled(),
            Err(DiagnosticError::ValidationCancelled)
        );
    }
}
