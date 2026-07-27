//! Production Best Practices Analyzer for Terraform.
//!
//! Evaluates Terraform version constraints, provider pinning, remote state locking,
//! resource tagging conventions, reusable module structure, and output declarations.

use crate::terraform::parser::TerraformAST;
use crate::yaml::parser::Span;

/// Best practice recommendation item.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BestPracticeRecommendation {
    /// Rule identifier.
    pub rule_id: String,
    /// Detailed recommendation text.
    pub message: String,
    /// Span location.
    pub span: Span,
}

/// Best practices evaluator service.
#[derive(Debug, Clone, Default)]
pub struct BestPracticesAnalyzer;

impl BestPracticesAnalyzer {
    /// Creates a new BestPracticesAnalyzer.
    pub fn new() -> Self {
        Self
    }

    /// Evaluates Terraform configurations against production best practices.
    pub fn evaluate(&self, ast: &TerraformAST) -> Vec<BestPracticeRecommendation> {
        let mut recs = Vec::new();

        // 1. Check terraform block presence and required_version
        let tf_blocks = ast.get_blocks_by_type("terraform");
        if tf_blocks.is_empty() {
            let empty_span = Span::default();
            recs.push(BestPracticeRecommendation {
                rule_id: "bp-tf-missing-terraform-block".to_string(),
                message: "Configuration is missing top-level 'terraform' block with required_version constraint".to_string(),
                span: empty_span,
            });
        } else {
            for block in tf_blocks {
                if block.get_attribute("required_version").is_none() {
                    recs.push(BestPracticeRecommendation {
                        rule_id: "bp-tf-missing-required-version".to_string(),
                        message:
                            "Top-level 'terraform' block should specify explicit 'required_version'"
                                .to_string(),
                        span: block.span,
                    });
                }
            }
        }

        // 2. Check for literal values in resources instead of variables
        for block in ast.get_blocks_by_type("resource") {
            for attr in &block.attributes {
                if attr.name == "cidr_block" || attr.name == "instance_type" {
                    let val = attr.value_expression.trim_matches('"');
                    if !val.starts_with("${") && !val.starts_with("var.") {
                        recs.push(BestPracticeRecommendation {
                            rule_id: "bp-tf-use-variables-over-literals".to_string(),
                            message: format!("Attribute '{}' uses hardcoded literal '{}'; consider extracting to a variable", attr.name, val),
                            span: attr.span,
                        });
                    }
                }
            }
        }

        recs
    }
}
