//! Terraform Network Security Analyzer.
//!
//! Audits Security Group ingress rules, CIDR block `0.0.0.0/0`, and exposed management/database ports.

use crate::terraform::parser::TerraformAST;
use crate::terraform::security::{TerraformSecurityFinding, TerraformSecuritySeverity};

/// Network security analyzer.
#[derive(Debug, Clone, Default)]
pub struct NetworkSecurityAnalyzer;

impl NetworkSecurityAnalyzer {
    /// Creates a new NetworkSecurityAnalyzer.
    pub fn new() -> Self {
        Self
    }

    /// Audits network security groups and ingress CIDR rules across an AST.
    pub fn audit_networking(&self, ast: &TerraformAST) -> Vec<TerraformSecurityFinding> {
        let mut findings = Vec::new();

        for block in &ast.blocks {
            if block.block_type != "resource" {
                continue;
            }

            let res_type = block.first_label().unwrap_or("");
            if matches!(
                res_type,
                "aws_security_group" | "aws_security_group_rule" | "azurerm_network_security_rule"
            ) {
                // Check for 0.0.0.0/0 in cidr_blocks attribute or nested ingress block
                for attr in &block.attributes {
                    if attr.name == "cidr_blocks" && attr.value_expression.contains("0.0.0.0/0") {
                        findings.push(TerraformSecurityFinding {
                            rule_id: "sec-tf-open-sg".to_string(),
                            message: format!("Security rule in '{res_type}' allows unrestricted ingress from 0.0.0.0/0"),
                            severity: TerraformSecuritySeverity::High,
                            span: attr.span,
                        });
                    }
                }

                for nested in &block.nested_blocks {
                    if nested.block_type == "ingress" {
                        if let Some(cidr) = nested.get_attribute("cidr_blocks") {
                            if cidr.value_expression.contains("0.0.0.0/0") {
                                let from_port = nested
                                    .get_attribute("from_port")
                                    .map(|a| a.value_expression.as_str())
                                    .unwrap_or("");
                                let msg = if from_port == "22" || from_port == "3389" {
                                    format!("Security group ingress exposes critical port {from_port} to 0.0.0.0/0")
                                } else {
                                    "Security group ingress rule allows unrestricted access from 0.0.0.0/0".to_string()
                                };

                                findings.push(TerraformSecurityFinding {
                                    rule_id: "sec-tf-open-sg".to_string(),
                                    message: msg,
                                    severity: TerraformSecuritySeverity::Critical,
                                    span: nested.span,
                                });
                            }
                        }
                    }
                }
            }
        }

        findings
    }
}
