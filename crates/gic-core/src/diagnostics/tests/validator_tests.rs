//! Validator trait, GenericRuleValidator, and CoreSyntaxValidator tests.

use crate::diagnostics::validator::{CoreSyntaxValidator, ValidationContext, Validator};
use crate::parser::diagnostics::Diagnostic as ParseDiag;
use crate::parser::language::LanguageId;
use crate::parser::node::{NodeKind, SyntaxNode};
use crate::parser::position::{Position as ParsePos, TextRange as ParseRange};
use crate::parser::tree::SyntaxTree;

#[test]
fn test_core_syntax_validator() {
    let p_start = ParsePos::new(0, 0, 0);
    let p_end = ParsePos::new(0, 5, 5);
    let range = ParseRange::new(p_start, p_end);
    let p_diag = ParseDiag::error(range, "Unexpected character ':'", "yaml-parser");

    let root = SyntaxNode::new(NodeKind::Document, "document", range, Vec::new());
    let tree = SyntaxTree::new(LanguageId::Yaml, root, Vec::new(), vec![p_diag], 12345, 10);

    let validator = CoreSyntaxValidator;
    let ctx = ValidationContext::new("foo: : bar", LanguageId::Yaml);
    let diags = validator.validate(&tree, &ctx).unwrap();

    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].message, "Unexpected character ':'");
    assert_eq!(diags[0].rule_name, "yaml-parser");
    assert_eq!(diags[0].line, 1);
    assert_eq!(diags[0].column, 1);
}
