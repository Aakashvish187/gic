//! Terraform Expression and String Interpolation Evaluator.
//!
//! Parses `${var.name}`, `${local.name}`, `${aws_s3_bucket.b.arn}`, `${module.vpc.id}`,
//! and expression functions (`concat`, `length`, `element`, `join`, `merge`, `lookup`, `coalesce`).

use crate::yaml::parser::Span;

/// Interpolation token classification.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum InterpolationKind {
    /// Variable reference (`var.foo`).
    Variable(String),
    /// Local value reference (`local.bar`).
    Local(String),
    /// Resource attribute reference (`aws_s3_bucket.b.id`).
    Resource {
        resource_type: String,
        name: String,
        attribute: String,
    },
    /// Module output reference (`module.vpc.vpc_id`).
    ModuleOutput {
        module_name: String,
        output_name: String,
    },
    /// Built-in function call (`concat(...)`, `length(...)`).
    FunctionCall {
        function_name: String,
        raw_arguments: String,
    },
}

/// Parsed expression interpolation item.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InterpolationExpression {
    /// Full expression text.
    pub raw: String,
    /// Classified token kind.
    pub kind: InterpolationKind,
    /// Source span location.
    pub span: Span,
}

/// Interpolation parser and expression analyzer.
#[derive(Debug, Clone, Default)]
pub struct InterpolationAnalyzer;

impl InterpolationAnalyzer {
    /// Creates a new InterpolationAnalyzer.
    pub fn new() -> Self {
        Self
    }

    /// Extracts all `${...}` interpolation expressions from a raw string or attribute value.
    pub fn parse_interpolations(&self, raw: &str, span: Span) -> Vec<InterpolationExpression> {
        let mut results = Vec::new();

        let mut curr = raw;
        while let Some(start_idx) = curr.find("${") {
            let rest = &curr[start_idx + 2..];
            if let Some(end_idx) = rest.find('}') {
                let expr_body = rest[..end_idx].trim();
                if let Some(kind) = classify_expression(expr_body) {
                    results.push(InterpolationExpression {
                        raw: format!("${{{expr_body}}}"),
                        kind,
                        span,
                    });
                }
                curr = &rest[end_idx + 1..];
            } else {
                break;
            }
        }

        if results.is_empty() {
            if let Some(kind) = classify_expression(raw.trim()) {
                results.push(InterpolationExpression {
                    raw: raw.to_string(),
                    kind,
                    span,
                });
            }
        }

        results
    }
}

fn classify_expression(expr: &str) -> Option<InterpolationKind> {
    if expr.is_empty() {
        return None;
    }

    if expr.starts_with("var.") {
        let var_name = expr.trim_start_matches("var.").to_string();
        return Some(InterpolationKind::Variable(var_name));
    }

    if expr.starts_with("local.") {
        let local_name = expr.trim_start_matches("local.").to_string();
        return Some(InterpolationKind::Local(local_name));
    }

    if expr.starts_with("module.") {
        let rest = expr.trim_start_matches("module.");
        if let Some((mod_name, out_name)) = rest.split_once('.') {
            return Some(InterpolationKind::ModuleOutput {
                module_name: mod_name.to_string(),
                output_name: out_name.to_string(),
            });
        }
    }

    if let Some((fn_name, args_part)) = expr.split_once('(') {
        if args_part.ends_with(')') {
            let args = args_part.trim_end_matches(')').trim().to_string();
            return Some(InterpolationKind::FunctionCall {
                function_name: fn_name.trim().to_string(),
                raw_arguments: args,
            });
        }
    }

    let parts: Vec<&str> = expr.split('.').collect();
    if parts.len() >= 3 && !parts[0].contains(' ') && !parts[1].contains(' ') {
        return Some(InterpolationKind::Resource {
            resource_type: parts[0].to_string(),
            name: parts[1].to_string(),
            attribute: parts[2..].join("."),
        });
    }

    None
}
