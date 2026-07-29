//! Central Terraform Validation Engine.
//!
//! Coordinates HCL AST analysis, resource extraction, provider validation, variable and output analysis,
//! module source validation, IaC security audits, networking checks, best practice evaluations,
//! reference resolution, and dependency graph cycle detection.

use std::collections::HashSet;

use crate::terraform::backend::BackendValidator;
use crate::terraform::best_practices::BestPracticesAnalyzer;
use crate::terraform::dependencies::DependencyAnalyzer;
use crate::terraform::modules::ModuleValidator;
use crate::terraform::networking::NetworkSecurityAnalyzer;
use crate::terraform::outputs::OutputValidator;
use crate::terraform::parser::{TerraformAST, TerraformParser};
use crate::terraform::providers::ProviderValidator;
use crate::terraform::references::ReferenceResolver;
use crate::terraform::resources::ResourceExtractor;
use crate::terraform::security::{TerraformSecurityAnalyzer, TerraformSecuritySeverity};
use crate::terraform::variables::VariableValidator;
use crate::yaml::parser::{Position, Span};

/// Diagnostic severity for Terraform validation findings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TerraformSeverity {
    Error,
    Warning,
    Info,
    Hint,
}

/// Primary diagnostic item produced by `TerraformValidator`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerraformDiagnostic {
    /// Rule identifier.
    pub rule_id: String,
    /// Human-readable message.
    pub message: String,
    /// Severity level.
    pub severity: TerraformSeverity,
    /// Target span location in source code.
    pub span: Span,
    /// Quick-fix proposal.
    pub quick_fix: Option<(String, String)>,
}

/// Central Terraform validator engine.
#[derive(Debug, Clone, Default)]
pub struct TerraformValidator {
    parser: TerraformParser,
    resource_extractor: ResourceExtractor,
    provider_validator: ProviderValidator,
    variable_validator: VariableValidator,
    output_validator: OutputValidator,
    module_validator: ModuleValidator,
    backend_validator: BackendValidator,
    security_analyzer: TerraformSecurityAnalyzer,
    network_analyzer: NetworkSecurityAnalyzer,
    best_practices_analyzer: BestPracticesAnalyzer,
    reference_resolver: ReferenceResolver,
    dependency_analyzer: DependencyAnalyzer,
}

impl TerraformValidator {
    /// Creates a new TerraformValidator.
    pub fn new() -> Self {
        Self::default()
    }

    /// Validates raw `.tf` source code string.
    pub fn validate_source(&self, source: &str) -> Vec<TerraformDiagnostic> {
        let mut diagnostics = Vec::new();
        let ast = match self.parser.parse(source) {
            Ok(ast) => ast,
            Err(err) => {
                let empty_pos = Position::new(1, 1, 0);
                diagnostics.push(TerraformDiagnostic {
                    rule_id: "tf-syntax-error".to_string(),
                    message: err.to_string(),
                    severity: TerraformSeverity::Error,
                    span: Span::new(empty_pos, empty_pos),
                    quick_fix: None,
                });
                return diagnostics;
            }
        };

        self.validate_ast(&ast, &mut diagnostics);
        diagnostics
    }

    /// Validates a parsed `TerraformAST`.
    pub fn validate_ast(&self, ast: &TerraformAST, diagnostics: &mut Vec<TerraformDiagnostic>) {
        let mut extracted_resources = Vec::new();
        let mut declared_var_names = HashSet::new();
        let mut used_var_names = HashSet::new();
        let mut declared_output_names = HashSet::new();

        for block in &ast.blocks {
            // 1. Resource extraction
            if matches!(block.block_type.as_str(), "resource" | "data") {
                if let Some(res) = self.resource_extractor.extract_resource(block) {
                    extracted_resources.push(res);
                }
            }

            // 2. Variable declarations & tracking
            if block.block_type == "variable" {
                if let Some(var_decl) = self.variable_validator.extract_variable(block) {
                    if !declared_var_names.insert(var_decl.name.clone()) {
                        diagnostics.push(TerraformDiagnostic {
                            rule_id: "tf-duplicate-variable".to_string(),
                            message: format!("Duplicate variable declaration '{}'", var_decl.name),
                            severity: TerraformSeverity::Error,
                            span: block.span,
                            quick_fix: None,
                        });
                    }
                }
            }

            // 3. Output declarations & tracking
            if block.block_type == "output" {
                if let Some(out_decl) = self.output_validator.extract_output(block) {
                    if !declared_output_names.insert(out_decl.name.clone()) {
                        diagnostics.push(TerraformDiagnostic {
                            rule_id: "tf-duplicate-output".to_string(),
                            message: format!("Duplicate output declaration '{}'", out_decl.name),
                            severity: TerraformSeverity::Error,
                            span: block.span,
                            quick_fix: None,
                        });
                    }
                }
            }

            // 4. Module calls
            if block.block_type == "module" {
                if let Some(mod_call) = self.module_validator.extract_module(block) {
                    if mod_call.source.is_empty() {
                        diagnostics.push(TerraformDiagnostic {
                            rule_id: "tf-module-missing-source".to_string(),
                            message: format!(
                                "Module call '{}' is missing required 'source' attribute",
                                mod_call.name
                            ),
                            severity: TerraformSeverity::Error,
                            span: block.span,
                            quick_fix: None,
                        });
                    }
                }
            }

            // 5. Provider configuration check
            if block.block_type == "provider" {
                if let Some(prov) = self.provider_validator.extract_provider_config(block) {
                    if prov.version.is_none() {
                        diagnostics.push(TerraformDiagnostic {
                            rule_id: "tf-provider-unpinned-version".to_string(),
                            message: format!(
                                "Provider '{}' should specify version constraint",
                                prov.name
                            ),
                            severity: TerraformSeverity::Warning,
                            span: block.span,
                            quick_fix: None,
                        });
                    }
                }
            }

            // 5. Backend check inside terraform block
            if block.block_type == "terraform" {
                for nested in block.get_nested_blocks("backend") {
                    if let Some(be) = self.backend_validator.extract_backend(nested) {
                        if !be.has_state_locking && be.backend_type == "s3" {
                            diagnostics.push(TerraformDiagnostic {
                                rule_id: "tf-backend-no-locking".to_string(),
                                message: "S3 backend does not configure state locking (missing 'dynamodb_table')".to_string(),
                                severity: TerraformSeverity::Warning,
                                span: nested.span,
                                quick_fix: None,
                            });
                        }
                    }
                }
            }

            // Track variable usage in attribute expressions
            for attr in &block.attributes {
                for token in attr
                    .value_expression
                    .split(|c: char| c.is_whitespace() || c == '"' || c == '\'')
                {
                    if token.starts_with("var.") || token.starts_with("${var.") {
                        let var_name = token
                            .trim_start_matches("${var.")
                            .trim_start_matches("var.")
                            .trim_end_matches('}')
                            .split('.')
                            .next()
                            .unwrap_or("");
                        if !var_name.is_empty() {
                            used_var_names.insert(var_name.to_string());
                        }
                    }
                }
            }
        }

        // 6. Check unused variables
        for var_name in &declared_var_names {
            if !used_var_names.contains(var_name) {
                let span = ast
                    .get_blocks_by_type("variable")
                    .iter()
                    .find(|b| b.first_label() == Some(var_name.as_str()))
                    .map(|b| b.span)
                    .unwrap_or_default();

                diagnostics.push(TerraformDiagnostic {
                    rule_id: "tf-unused-variable".to_string(),
                    message: format!("Variable '{var_name}' is declared but never referenced"),
                    severity: TerraformSeverity::Warning,
                    span,
                    quick_fix: None,
                });
            }
        }

        // 7. Reference resolution
        let symbol_table = self
            .reference_resolver
            .build_symbol_table(std::slice::from_ref(ast));
        for (unresolved_ref, span) in self
            .reference_resolver
            .find_unresolved_references(ast, &symbol_table)
        {
            diagnostics.push(TerraformDiagnostic {
                rule_id: "tf-unresolved-reference".to_string(),
                message: format!(
                    "Reference '{unresolved_ref}' cannot be resolved in current scope"
                ),
                severity: TerraformSeverity::Error,
                span,
                quick_fix: None,
            });
        }

        // 8. Dependency graph cycle detection
        let dep_graph = self.dependency_analyzer.build_graph(&extracted_resources);
        let cycles = self.dependency_analyzer.find_cycles(&dep_graph);
        for cycle in cycles {
            diagnostics.push(TerraformDiagnostic {
                rule_id: "tf-circular-dependency".to_string(),
                message: format!(
                    "Circular dependency detected between resources: {}",
                    cycle.join(" -> ")
                ),
                severity: TerraformSeverity::Error,
                span: Span::default(),
                quick_fix: None,
            });
        }

        // 9. IaC Security Audit
        for finding in self.security_analyzer.audit_ast(ast) {
            let severity = match finding.severity {
                TerraformSecuritySeverity::Critical | TerraformSecuritySeverity::High => {
                    TerraformSeverity::Error
                }
                TerraformSecuritySeverity::Medium => TerraformSeverity::Warning,
                TerraformSecuritySeverity::Low => TerraformSeverity::Info,
            };
            diagnostics.push(TerraformDiagnostic {
                rule_id: finding.rule_id,
                message: finding.message,
                severity,
                span: finding.span,
                quick_fix: None,
            });
        }

        // 10. Networking Security Audit
        for finding in self.network_analyzer.audit_networking(ast) {
            let severity = match finding.severity {
                TerraformSecuritySeverity::Critical | TerraformSecuritySeverity::High => {
                    TerraformSeverity::Error
                }
                TerraformSecuritySeverity::Medium => TerraformSeverity::Warning,
                TerraformSecuritySeverity::Low => TerraformSeverity::Info,
            };
            diagnostics.push(TerraformDiagnostic {
                rule_id: finding.rule_id,
                message: finding.message,
                severity,
                span: finding.span,
                quick_fix: None,
            });
        }

        // 11. Production Best Practices
        for rec in self.best_practices_analyzer.evaluate(ast) {
            diagnostics.push(TerraformDiagnostic {
                rule_id: rec.rule_id,
                message: rec.message,
                severity: TerraformSeverity::Hint,
                span: rec.span,
                quick_fix: None,
            });
        }
    }
}
