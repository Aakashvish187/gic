//! Terraform Variable Declaration and Validation Engine.
//!
//! Extracts `variable` blocks, parses default values, type constraints, descriptions,
//! sensitivity flags, and identifies unused variable declarations.

use crate::terraform::parser::HclBlock;
use crate::yaml::parser::Span;

/// Extracted `variable` definition node.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct VariableDeclaration {
    /// Variable name identifier (e.g., `"environment"`).
    pub name: String,
    /// Default value expression if specified.
    pub default_value: Option<String>,
    /// Declared type constraint (e.g. `"string"`, `"list(string)"`, `"map(any)"`).
    pub type_constraint: Option<String>,
    /// Variable documentation description.
    pub description: Option<String>,
    /// Sensitive flag (`sensitive = true`).
    pub is_sensitive: bool,
    /// Nullable flag (`nullable = false`).
    pub is_nullable: bool,
    /// Source span of the variable block.
    pub span: Span,
}

/// Variable declaration validator.
#[derive(Debug, Clone, Default)]
pub struct VariableValidator;

impl VariableValidator {
    /// Creates a new VariableValidator.
    pub fn new() -> Self {
        Self
    }

    /// Extracts `VariableDeclaration` from an HCL `variable` block.
    pub fn extract_variable(&self, block: &HclBlock) -> Option<VariableDeclaration> {
        if block.block_type != "variable" {
            return None;
        }

        let name = block.first_label()?.to_string();

        let mut default_value = None;
        let mut type_constraint = None;
        let mut description = None;
        let mut is_sensitive = false;
        let mut is_nullable = true;

        for attr in &block.attributes {
            match attr.name.as_str() {
                "default" => default_value = Some(attr.value_expression.clone()),
                "type" => type_constraint = Some(attr.value_expression.clone()),
                "description" => {
                    description = Some(attr.value_expression.trim_matches('"').to_string())
                }
                "sensitive" => is_sensitive = attr.value_expression.eq_ignore_ascii_case("true"),
                "nullable" => is_nullable = !attr.value_expression.eq_ignore_ascii_case("false"),
                _ => {}
            }
        }

        Some(VariableDeclaration {
            name,
            default_value,
            type_constraint,
            description,
            is_sensitive,
            is_nullable,
            span: block.span,
        })
    }
}
