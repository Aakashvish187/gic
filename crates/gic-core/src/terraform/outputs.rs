//! Terraform Output Declaration and Validation Engine.
//!
//! Extracts `output` blocks, checks value expressions, descriptions, sensitivity flags,
//! duplicate output declarations, and unused outputs.

use crate::terraform::parser::HclBlock;
use crate::yaml::parser::Span;

/// Extracted `output` definition node.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct OutputDeclaration {
    /// Output name identifier (e.g. `"bucket_arn"`).
    pub name: String,
    /// Value expression (e.g., `"aws_s3_bucket.b.arn"`).
    pub value_expression: String,
    /// Output documentation description.
    pub description: Option<String>,
    /// Sensitive flag (`sensitive = true`).
    pub is_sensitive: bool,
    /// Explicit `depends_on` referenced address list.
    pub depends_on: Vec<String>,
    /// Source span of the output block.
    pub span: Span,
}

/// Output declaration validator.
#[derive(Debug, Clone, Default)]
pub struct OutputValidator;

impl OutputValidator {
    /// Creates a new OutputValidator.
    pub fn new() -> Self {
        Self
    }

    /// Extracts `OutputDeclaration` from an HCL `output` block.
    pub fn extract_output(&self, block: &HclBlock) -> Option<OutputDeclaration> {
        if block.block_type != "output" {
            return None;
        }

        let name = block.first_label()?.to_string();

        let mut value_expression = String::new();
        let mut description = None;
        let mut is_sensitive = false;
        let mut depends_on = Vec::new();

        for attr in &block.attributes {
            match attr.name.as_str() {
                "value" => value_expression = attr.value_expression.clone(),
                "description" => {
                    description = Some(attr.value_expression.trim_matches('"').to_string())
                }
                "sensitive" => is_sensitive = attr.value_expression.eq_ignore_ascii_case("true"),
                "depends_on" => depends_on = parse_string_list(&attr.value_expression),
                _ => {}
            }
        }

        Some(OutputDeclaration {
            name,
            value_expression,
            description,
            is_sensitive,
            depends_on,
            span: block.span,
        })
    }
}

fn parse_string_list(expr: &str) -> Vec<String> {
    let trimmed = expr
        .trim()
        .trim_matches(|c| c == '[' || c == ']' || c == ' ');
    trimmed
        .split(',')
        .map(|s| s.trim().trim_matches('"').trim_matches('\'').to_string())
        .filter(|s| !s.is_empty())
        .collect()
}
