//! Terraform Module Call and Reusability Validator.
//!
//! Extracts `module` blocks, parses module `source` (Git, Local, Registry),
//! `version` constraints, input arguments, `count`, `for_each`, and `providers` passing.

use std::collections::HashMap;

use crate::terraform::parser::HclBlock;
use crate::yaml::parser::Span;

/// Variant indicating the origin type of a Terraform module source.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ModuleSourceKind {
    /// Local file path (e.g. `"./modules/vpc"`).
    #[default]
    Local,
    /// Terraform Registry (e.g. `"terraform-aws-modules/vpc/aws"`).
    Registry,
    /// Git repository URL (e.g. `"git::https://example.com/repo.git"`).
    Git,
    /// S3 or HTTP bucket URL.
    Url,
}

/// Extracted `module` call node.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ModuleCall {
    /// Module local identifier name (e.g., `"vpc"`).
    pub name: String,
    /// Raw `source` attribute string.
    pub source: String,
    /// Classification of the module source origin.
    pub source_kind: ModuleSourceKind,
    /// Version constraint string if declared.
    pub version_constraint: Option<String>,
    /// Passed arguments mapped by parameter name.
    pub arguments: HashMap<String, String>,
    /// Source span of the block.
    pub span: Span,
}

/// Module call validator.
#[derive(Debug, Clone, Default)]
pub struct ModuleValidator;

impl ModuleValidator {
    /// Creates a new ModuleValidator.
    pub fn new() -> Self {
        Self
    }

    /// Extracts `ModuleCall` from an HCL `module` block.
    pub fn extract_module(&self, block: &HclBlock) -> Option<ModuleCall> {
        if block.block_type != "module" {
            return None;
        }

        let name = block.first_label()?.to_string();

        let mut source = String::new();
        let mut version_constraint = None;
        let mut arguments = HashMap::new();

        for attr in &block.attributes {
            match attr.name.as_str() {
                "source" => source = attr.value_expression.trim_matches('"').to_string(),
                "version" => {
                    version_constraint = Some(attr.value_expression.trim_matches('"').to_string())
                }
                _ => {
                    arguments.insert(attr.name.clone(), attr.value_expression.clone());
                }
            }
        }

        let source_kind = classify_module_source(&source);

        Some(ModuleCall {
            name,
            source,
            source_kind,
            version_constraint,
            arguments,
            span: block.span,
        })
    }
}

fn classify_module_source(source: &str) -> ModuleSourceKind {
    if source.starts_with('.') || source.starts_with('/') {
        ModuleSourceKind::Local
    } else if source.starts_with("git::") || source.starts_with("github.com") {
        ModuleSourceKind::Git
    } else if source.starts_with("http://")
        || source.starts_with("https://")
        || source.starts_with("s3::")
    {
        ModuleSourceKind::Url
    } else {
        ModuleSourceKind::Registry
    }
}
