//! Terraform Symbol Reference Resolver.
//!
//! Tracks declared symbols (`var.*`, `local.*`, `resource.*`, `data.*`, `module.*`),
//! resolves cross-file references, and detects undefined variable, output, or resource references.

use std::collections::HashSet;

use crate::terraform::interpolation::{InterpolationAnalyzer, InterpolationKind};
use crate::terraform::parser::TerraformAST;

/// Scope symbol table of declared identifiers in a Terraform workspace.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SymbolTable {
    /// Declared variable names (`"var.name"`).
    pub declared_variables: HashSet<String>,
    /// Declared local names (`"local.name"`).
    pub declared_locals: HashSet<String>,
    /// Declared resource addresses (`"aws_s3_bucket.b"`, `"data.aws_ami.ubuntu"`).
    pub declared_resources: HashSet<String>,
    /// Declared module names (`"module.vpc"`).
    pub declared_modules: HashSet<String>,
    /// Declared output names (`"output.arn"`).
    pub declared_outputs: HashSet<String>,
}

/// Symbol reference resolver engine.
#[derive(Debug, Clone, Default)]
pub struct ReferenceResolver {
    interpolation_analyzer: InterpolationAnalyzer,
}

impl ReferenceResolver {
    /// Creates a new ReferenceResolver.
    pub fn new() -> Self {
        Self {
            interpolation_analyzer: InterpolationAnalyzer::new(),
        }
    }

    /// Builds the symbol table from a collection of parsed `TerraformAST` files.
    pub fn build_symbol_table(&self, asts: &[TerraformAST]) -> SymbolTable {
        let mut table = SymbolTable::default();

        for ast in asts {
            for block in &ast.blocks {
                match block.block_type.as_str() {
                    "variable" => {
                        if let Some(lbl) = block.first_label() {
                            table.declared_variables.insert(lbl.to_string());
                        }
                    }
                    "locals" => {
                        for attr in &block.attributes {
                            table.declared_locals.insert(attr.name.clone());
                        }
                    }
                    "resource" => {
                        if let (Some(res_type), Some(res_name)) =
                            (block.first_label(), block.second_label())
                        {
                            table
                                .declared_resources
                                .insert(format!("{res_type}.{res_name}"));
                        }
                    }
                    "data" => {
                        if let (Some(res_type), Some(res_name)) =
                            (block.first_label(), block.second_label())
                        {
                            table
                                .declared_resources
                                .insert(format!("data.{res_type}.{res_name}"));
                        }
                    }
                    "module" => {
                        if let Some(lbl) = block.first_label() {
                            table.declared_modules.insert(lbl.to_string());
                        }
                    }
                    "output" => {
                        if let Some(lbl) = block.first_label() {
                            table.declared_outputs.insert(lbl.to_string());
                        }
                    }
                    _ => {}
                }
            }
        }

        table
    }

    /// Validates attribute interpolation references against the symbol table.
    pub fn find_unresolved_references(
        &self,
        ast: &TerraformAST,
        table: &SymbolTable,
    ) -> Vec<(String, crate::yaml::parser::Span)> {
        let mut unresolved = Vec::new();

        for block in &ast.blocks {
            for attr in &block.attributes {
                let exprs = self
                    .interpolation_analyzer
                    .parse_interpolations(&attr.value_expression, attr.span);
                for expr in exprs {
                    match expr.kind {
                        InterpolationKind::Variable(ref v) => {
                            let clean_name = v.split('.').next().unwrap_or(v);
                            if !table.declared_variables.contains(clean_name) {
                                unresolved.push((format!("var.{clean_name}"), expr.span));
                            }
                        }
                        InterpolationKind::Local(ref l) => {
                            let clean_name = l.split('.').next().unwrap_or(l);
                            if !table.declared_locals.contains(clean_name) {
                                unresolved.push((format!("local.{clean_name}"), expr.span));
                            }
                        }
                        InterpolationKind::Resource {
                            ref resource_type,
                            ref name,
                            ..
                        } => {
                            let addr = format!("{resource_type}.{name}");
                            if !table.declared_resources.contains(&addr)
                                && !addr.starts_with("data.")
                            {
                                unresolved.push((addr, expr.span));
                            }
                        }
                        InterpolationKind::ModuleOutput {
                            ref module_name, ..
                        }
                            if !table.declared_modules.contains(module_name) => {
                                unresolved.push((format!("module.{module_name}"), expr.span));
                            }
                        _ => {}
                    }
                }
            }
        }

        unresolved
    }
}
