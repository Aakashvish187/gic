//! ValidationEngine integration and execution unit tests.

use crate::diagnostics::diagnostic::Diagnostic;
use crate::diagnostics::engine::ValidationEngine;
use crate::diagnostics::errors::DiagnosticError;
use crate::diagnostics::position::DiagnosticPosition;
use crate::diagnostics::range::DiagnosticRange;
use crate::diagnostics::rule::{Rule, RuleCategory, RuleMetadata, RulePriority};
use crate::diagnostics::severity::DiagnosticLevel;
use crate::diagnostics::validator::ValidationContext;
use crate::parser::language::LanguageId;
use crate::parser::node::{NodeKind, SyntaxNode};
use crate::parser::position::{Position, TextRange};
use crate::parser::tree::SyntaxTree;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

struct MockSecurityRule {
    meta: RuleMetadata,
}

impl MockSecurityRule {
    fn new() -> Self {
        Self {
            meta: RuleMetadata::new(
                "GIC-SEC-001",
                "NoHardcodedPasswords",
                RuleCategory::Security,
                DiagnosticLevel::Security,
            )
            .with_priority(RulePriority::High)
            .with_languages(vec![LanguageId::Yaml, LanguageId::Dockerfile]),
        }
    }
}

impl Rule for MockSecurityRule {
    fn metadata(&self) -> &RuleMetadata {
        &self.meta
    }

    fn evaluate(
        &self,
        _tree: &SyntaxTree,
        ctx: &ValidationContext,
    ) -> Result<Vec<Diagnostic>, DiagnosticError> {
        let mut diags = Vec::new();
        if ctx.source_text.contains("password: 123") {
            let p1 = DiagnosticPosition::new(1, 1, 0);
            let p2 = DiagnosticPosition::new(1, 13, 12);
            let range = DiagnosticRange::new(p1, p2);

            let d = Diagnostic::new(
                DiagnosticLevel::Security,
                "Hardcoded plaintext password detected",
                range,
                self.meta.name.clone(),
                ctx.language.clone(),
            )
            .with_description(
                "Secrets should be stored in environment variables or vault references.",
            );

            diags.push(d);
        }
        Ok(diags)
    }
}

#[test]
fn test_engine_full_validation_cycle() {
    let engine = ValidationEngine::new();
    engine.rule_registry().register(MockSecurityRule::new());

    let root = SyntaxNode::new(
        NodeKind::Document,
        "document",
        TextRange::empty(Position::zero()),
        Vec::new(),
    );
    let tree = SyntaxTree::new(LanguageId::Yaml, root, Vec::new(), Vec::new(), 112233, 50);

    let source_bad = "password: 123\n";
    let diags = engine
        .validate("config.yaml", &tree, source_bad, None)
        .unwrap();

    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].severity, DiagnosticLevel::Security);
    assert_eq!(diags[0].rule_name, "NoHardcodedPasswords");
    assert!(diags[0].description.is_some());

    // Clean source test
    let source_good = "password: ${ENV_PASS}\n";
    let tree_good = SyntaxTree::new(
        LanguageId::Yaml,
        SyntaxNode::new(
            NodeKind::Document,
            "doc",
            TextRange::empty(Position::zero()),
            Vec::new(),
        ),
        Vec::new(),
        Vec::new(),
        445566,
        50,
    );
    let diags_clean = engine
        .validate("config_good.yaml", &tree_good, source_good, None)
        .unwrap();
    assert!(diags_clean.is_empty());
}

#[test]
fn test_engine_cancellation_signal() {
    let engine = ValidationEngine::new();
    engine.rule_registry().register(MockSecurityRule::new());

    let root = SyntaxNode::new(
        NodeKind::Document,
        "document",
        TextRange::empty(Position::zero()),
        Vec::new(),
    );
    let tree = SyntaxTree::new(LanguageId::Yaml, root, Vec::new(), Vec::new(), 778899, 50);

    let cancel_flag = Arc::new(AtomicBool::new(true));
    let result = engine.validate(
        "cancel_test.yaml",
        &tree,
        "password: 123\n",
        Some(cancel_flag),
    );

    assert_eq!(result, Err(DiagnosticError::ValidationCancelled));
}
