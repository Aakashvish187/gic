//! Terraform (HCL) AST Parser and Data Models.
//!
//! Scans raw `.tf` / `.hcl` source code, extracts top-level and nested HCL blocks,
//! parses attributes, expressions, string interpolations, and builds `TerraformAST`.

use crate::terraform::errors::TerraformResult;
use crate::yaml::parser::{Position, Span};

/// HCL attribute representation (`key = value`).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct HclAttribute {
    /// Attribute key identifier name.
    pub name: String,
    /// Raw unparsed value string expression.
    pub value_expression: String,
    /// Span of the attribute name.
    pub key_span: Span,
    /// Entire span of the key-value attribute.
    pub span: Span,
}

/// Structural block in Terraform configuration (`block_type "label1" "label2" { ... }`).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct HclBlock {
    /// Primary block keyword (e.g., `"resource"`, `"variable"`, `"module"`, `"provider"`).
    pub block_type: String,
    /// Block labels (e.g. `["aws_s3_bucket", "my_bucket"]`).
    pub labels: Vec<String>,
    /// Directly declared key-value attributes.
    pub attributes: Vec<HclAttribute>,
    /// Nested child blocks (e.g., `lifecycle`, `dynamic`, `provisioner`).
    pub nested_blocks: Vec<HclBlock>,
    /// Entire source span of the block.
    pub span: Span,
}

impl HclBlock {
    /// Returns the first label if present.
    pub fn first_label(&self) -> Option<&str> {
        self.labels.first().map(|s| s.as_str())
    }

    /// Returns the second label if present.
    pub fn second_label(&self) -> Option<&str> {
        self.labels.get(1).map(|s| s.as_str())
    }

    /// Finds an attribute by key name.
    pub fn get_attribute(&self, name: &str) -> Option<&HclAttribute> {
        self.attributes.iter().find(|attr| attr.name == name)
    }

    /// Finds nested child blocks by block type.
    pub fn get_nested_blocks(&self, block_type: &str) -> Vec<&HclBlock> {
        self.nested_blocks
            .iter()
            .filter(|b| b.block_type == block_type)
            .collect()
    }
}

/// Complete parsed AST of a Terraform configuration file.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TerraformAST {
    /// Top-level blocks declared in source code.
    pub blocks: Vec<HclBlock>,
    /// Original raw source text.
    pub source: String,
}

impl TerraformAST {
    /// Returns true if the AST contains no top-level blocks.
    pub fn is_empty(&self) -> bool {
        self.blocks.is_empty()
    }

    /// Returns all top-level blocks matching a specific block type (e.g., `"resource"`).
    pub fn get_blocks_by_type(&self, block_type: &str) -> Vec<&HclBlock> {
        self.blocks
            .iter()
            .filter(|b| b.block_type == block_type)
            .collect()
    }
}

/// High-performance zero-panic Terraform HCL parser.
#[derive(Debug, Clone, Default)]
pub struct TerraformParser;

impl TerraformParser {
    /// Creates a new TerraformParser.
    pub fn new() -> Self {
        Self
    }

    /// Parses raw `.tf` text source into a `TerraformAST`.
    pub fn parse(&self, source: &str) -> TerraformResult<TerraformAST> {
        let mut blocks = Vec::new();
        let lines: Vec<&str> = source.lines().collect();

        let mut idx = 0;
        while idx < lines.len() {
            let raw_line = lines[idx];
            let trimmed = strip_comments(raw_line).trim();

            if trimmed.is_empty() {
                idx += 1;
                continue;
            }

            // Check if line introduces a block: `block_type [label1] [label2] {`
            if is_block_header(trimmed) {
                let (block, next_idx) = parse_block(&lines, idx)?;
                blocks.push(block);
                idx = next_idx;
            } else {
                idx += 1;
            }
        }

        Ok(TerraformAST {
            blocks,
            source: source.to_string(),
        })
    }
}

fn strip_comments(line: &str) -> &str {
    if let Some(hash_pos) = line.find('#') {
        &line[..hash_pos]
    } else if let Some(slash_pos) = line.find("//") {
        &line[..slash_pos]
    } else {
        line
    }
}

fn is_block_header(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.ends_with('{')
        || (trimmed.contains('{') && !trimmed.contains('=') && !trimmed.contains("function"))
        || (trimmed.starts_with("terraform")
            || trimmed.starts_with("provider")
            || trimmed.starts_with("resource")
            || trimmed.starts_with("data")
            || trimmed.starts_with("module")
            || trimmed.starts_with("variable")
            || trimmed.starts_with("locals")
            || trimmed.starts_with("output")
            || trimmed.starts_with("backend")
            || trimmed.starts_with("moved")
            || trimmed.starts_with("import")
            || trimmed.starts_with("check")
            || trimmed.starts_with("lifecycle")
            || trimmed.starts_with("provisioner")
            || trimmed.starts_with("dynamic"))
}

fn parse_block(lines: &[&str], start_idx: usize) -> TerraformResult<(HclBlock, usize)> {
    let header_line = lines[start_idx];
    let (block_type, labels) = parse_header_tokens(header_line);

    let start_pos = Position::new(start_idx + 1, 1, 0);
    let mut attributes = Vec::new();
    let mut nested_blocks = Vec::new();

    let mut idx = start_idx + 1;
    let mut brace_depth = 1;

    while idx < lines.len() {
        let raw_line = lines[idx];
        let trimmed = strip_comments(raw_line).trim();

        if trimmed.is_empty() {
            idx += 1;
            continue;
        }

        let open_count = trimmed.chars().filter(|&c| c == '{').count();
        let close_count = trimmed.chars().filter(|&c| c == '}').count();

        if close_count > open_count && brace_depth == 1 && trimmed.ends_with('}') {
            let end_pos = Position::new(idx + 1, raw_line.len().max(1), 0);
            let span = Span::new(start_pos, end_pos);
            return Ok((
                HclBlock {
                    block_type,
                    labels,
                    attributes,
                    nested_blocks,
                    span,
                },
                idx + 1,
            ));
        }

        if is_block_header(trimmed) && trimmed.contains('{') {
            let (nested, next_idx) = parse_block(lines, idx)?;
            nested_blocks.push(nested);
            idx = next_idx;
            continue;
        }

        if let Some((attr_name, attr_val)) = parse_attribute_line(trimmed, idx + 1) {
            let attr_start = Position::new(idx + 1, 1, 0);
            let attr_end = Position::new(idx + 1, raw_line.len().max(1), 0);
            attributes.push(HclAttribute {
                name: attr_name,
                value_expression: attr_val,
                key_span: Span::new(attr_start, attr_end),
                span: Span::new(attr_start, attr_end),
            });
        }

        brace_depth = brace_depth + open_count - close_count;
        if brace_depth == 0 {
            let end_pos = Position::new(idx + 1, raw_line.len().max(1), 0);
            let span = Span::new(start_pos, end_pos);
            return Ok((
                HclBlock {
                    block_type,
                    labels,
                    attributes,
                    nested_blocks,
                    span,
                },
                idx + 1,
            ));
        }

        idx += 1;
    }

    let end_pos = Position::new(lines.len(), 1, 0);
    Ok((
        HclBlock {
            block_type,
            labels,
            attributes,
            nested_blocks,
            span: Span::new(start_pos, end_pos),
        },
        lines.len(),
    ))
}

fn parse_header_tokens(header: &str) -> (String, Vec<String>) {
    let cleaned = header.trim().trim_end_matches('{').trim();
    let tokens = tokenize_header_line(cleaned);

    if tokens.is_empty() {
        return (String::new(), Vec::new());
    }

    let block_type = tokens[0].clone();
    let labels = tokens[1..].to_vec();
    (block_type, labels)
}

fn tokenize_header_line(line: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut in_quotes = false;
    let mut current = String::new();

    for c in line.chars() {
        match c {
            '"' => {
                in_quotes = !in_quotes;
            }
            c if c.is_whitespace() && !in_quotes => {
                if !current.is_empty() {
                    tokens.push(current.clone());
                    current.clear();
                }
            }
            _ => {
                current.push(c);
            }
        }
    }

    if !current.is_empty() {
        tokens.push(current);
    }

    tokens
}

fn parse_attribute_line(line: &str, line_num: usize) -> Option<(String, String)> {
    if !line.contains('=') || line.starts_with('{') || line.starts_with('}') {
        return None;
    }

    let (key_part, val_part) = line.split_once('=')?;
    let key = key_part.trim();
    let val = val_part.trim();

    if key.is_empty() || key.contains(' ') {
        return None;
    }

    let _ = line_num;
    Some((key.to_string(), val.to_string()))
}
