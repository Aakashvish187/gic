//! Terraform Resource Model and Metadata Extractor.
//!
//! Extracts managed `resource` and data source `data` definitions from HCL blocks,
//! parses meta-arguments (`count`, `for_each`, `provider`, `lifecycle`, `depends_on`),
//! and models provider resource types.

use std::collections::HashMap;

use crate::terraform::parser::HclBlock;
use crate::yaml::parser::Span;

/// Variant indicating whether a resource is managed (`resource`) or a data source (`data`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ResourceMode {
    /// Managed resource (`resource "aws_s3_bucket" "b"`).
    #[default]
    Managed,
    /// Data source (`data "aws_ami" "ubuntu"`).
    Data,
}

/// Extracted Terraform resource representation.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TerraformResource {
    /// Resource mode (Managed vs Data).
    pub mode: ResourceMode,
    /// Provider resource type name (e.g. `"aws_s3_bucket"`, `"azurerm_resource_group"`).
    pub resource_type: String,
    /// Local resource name identifier (e.g. `"b"`, `"main"`).
    pub name: String,
    /// Full address (e.g. `"aws_s3_bucket.b"` or `"data.aws_ami.ubuntu"`).
    pub address: String,
    /// Attribute expressions mapped by key name.
    pub attributes: HashMap<String, String>,
    /// Meta-argument `count` expression if specified.
    pub count_expr: Option<String>,
    /// Meta-argument `for_each` expression if specified.
    pub for_each_expr: Option<String>,
    /// Meta-argument `provider` explicit alias.
    pub provider_alias: Option<String>,
    /// Explicit `depends_on` referenced address list.
    pub depends_on: Vec<String>,
    /// Source span of the block.
    pub span: Span,
}

/// Resource extractor service.
#[derive(Debug, Clone, Default)]
pub struct ResourceExtractor;

impl ResourceExtractor {
    /// Creates a new ResourceExtractor.
    pub fn new() -> Self {
        Self
    }

    /// Extracts `TerraformResource` items from an HCL block.
    pub fn extract_resource(&self, block: &HclBlock) -> Option<TerraformResource> {
        let mode = match block.block_type.as_str() {
            "resource" => ResourceMode::Managed,
            "data" => ResourceMode::Data,
            _ => return None,
        };

        let resource_type = block.first_label()?.to_string();
        let name = block.second_label()?.to_string();

        let address = match mode {
            ResourceMode::Managed => format!("{resource_type}.{name}"),
            ResourceMode::Data => format!("data.{resource_type}.{name}"),
        };

        let mut attributes = HashMap::new();
        let mut count_expr = None;
        let mut for_each_expr = None;
        let mut provider_alias = None;
        let mut depends_on = Vec::new();

        for attr in &block.attributes {
            match attr.name.as_str() {
                "count" => count_expr = Some(attr.value_expression.clone()),
                "for_each" => for_each_expr = Some(attr.value_expression.clone()),
                "provider" => provider_alias = Some(attr.value_expression.clone()),
                "depends_on" => depends_on = parse_string_list(&attr.value_expression),
                _ => {
                    attributes.insert(attr.name.clone(), attr.value_expression.clone());
                }
            }
        }

        Some(TerraformResource {
            mode,
            resource_type,
            name,
            address,
            attributes,
            count_expr,
            for_each_expr,
            provider_alias,
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
