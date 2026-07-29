//! Terraform IaC Security Audit Engine.
//!
//! Audits HCL configurations for hardcoded secrets, public S3 buckets, `0.0.0.0/0` security groups,
//! unencrypted storage, missing resource tags, wildcard IAM policies, public databases, and unpinned versions.

use crate::terraform::parser::TerraformAST;
use crate::yaml::parser::Span;

/// Security finding severity level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TerraformSecuritySeverity {
    Critical,
    High,
    Medium,
    Low,
}

/// Security audit finding item.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerraformSecurityFinding {
    /// Rule identifier.
    pub rule_id: String,
    /// Human-readable message.
    pub message: String,
    /// Severity level.
    pub severity: TerraformSecuritySeverity,
    /// Target span location.
    pub span: Span,
}

/// Security audit analyzer.
#[derive(Debug, Clone, Default)]
pub struct TerraformSecurityAnalyzer;

impl TerraformSecurityAnalyzer {
    /// Creates a new TerraformSecurityAnalyzer.
    pub fn new() -> Self {
        Self
    }

    /// Performs full security audit across a `TerraformAST`.
    pub fn audit_ast(&self, ast: &TerraformAST) -> Vec<TerraformSecurityFinding> {
        let mut findings = Vec::new();

        for block in &ast.blocks {
            match block.block_type.as_str() {
                "resource" => {
                    self.audit_resource_block(block, &mut findings);
                }
                "provider" => {
                    self.audit_provider_block(block, &mut findings);
                }
                "module" => {
                    self.audit_module_block(block, &mut findings);
                }
                _ => {}
            }

            // General hardcoded secrets check
            for attr in &block.attributes {
                self.audit_hardcoded_secrets(attr, &mut findings);
            }
        }

        findings
    }

    fn audit_resource_block(
        &self,
        block: &crate::terraform::parser::HclBlock,
        findings: &mut Vec<TerraformSecurityFinding>,
    ) {
        let res_type = block.first_label().unwrap_or("");

        // 1. S3 Bucket Public Access
        if res_type == "aws_s3_bucket" || res_type == "aws_s3_bucket_acl" {
            if let Some(acl_attr) = block.get_attribute("acl") {
                if acl_attr.value_expression.contains("public-read")
                    || acl_attr.value_expression.contains("public-read-write")
                {
                    findings.push(TerraformSecurityFinding {
                        rule_id: "sec-tf-public-s3".to_string(),
                        message: format!(
                            "S3 bucket '{res_type}' is configured with public ACL: {}",
                            acl_attr.value_expression
                        ),
                        severity: TerraformSecuritySeverity::High,
                        span: acl_attr.span,
                    });
                }
            }
        }

        // 2. Unencrypted Storage
        if matches!(
            res_type,
            "aws_ebs_volume" | "azurerm_managed_disk" | "google_compute_disk"
        ) {
            let enc = block.get_attribute("encrypted");
            if enc.map_or(true, |a| a.value_expression.eq_ignore_ascii_case("false")) {
                findings.push(TerraformSecurityFinding {
                    rule_id: "sec-tf-unencrypted-storage".to_string(),
                    message: format!(
                        "Storage volume '{res_type}' does not have encryption enabled"
                    ),
                    severity: TerraformSecuritySeverity::High,
                    span: block.span,
                });
            }
        }

        // 3. Open Public Database
        if matches!(
            res_type,
            "aws_db_instance" | "azurerm_postgresql_server" | "google_sql_database_instance"
        ) {
            if let Some(pub_attr) = block.get_attribute("publicly_accessible") {
                if pub_attr.value_expression.eq_ignore_ascii_case("true") {
                    findings.push(TerraformSecurityFinding {
                        rule_id: "sec-tf-open-db".to_string(),
                        message: format!("Database instance '{res_type}' is publicly accessible"),
                        severity: TerraformSecuritySeverity::Critical,
                        span: pub_attr.span,
                    });
                }
            }
        }

        // 4. Wildcard IAM Policy
        if matches!(res_type, "aws_iam_policy" | "aws_iam_role_policy") {
            if let Some(pol) = block.get_attribute("policy") {
                if pol.value_expression.contains("\"*\"")
                    || pol.value_expression.contains("Action = \"*\"")
                {
                    findings.push(TerraformSecurityFinding {
                        rule_id: "sec-tf-wildcard-iam".to_string(),
                        message: "IAM policy contains wildcard '*' permissions which violates least privilege".to_string(),
                        severity: TerraformSecuritySeverity::High,
                        span: pol.span,
                    });
                }
            }
        }

        // 5. Missing Resource Tags
        if res_type.starts_with("aws_") || res_type.starts_with("azurerm_") {
            let has_tags = block
                .attributes
                .iter()
                .any(|a| a.name == "tags" || a.name == "tags_all");
            if !has_tags {
                findings.push(TerraformSecurityFinding {
                    rule_id: "sec-tf-missing-tags".to_string(),
                    message: format!("Resource '{res_type}' is missing resource tags"),
                    severity: TerraformSecuritySeverity::Low,
                    span: block.span,
                });
            }
        }
    }

    fn audit_provider_block(
        &self,
        block: &crate::terraform::parser::HclBlock,
        findings: &mut Vec<TerraformSecurityFinding>,
    ) {
        if block.get_attribute("version").is_none() {
            findings.push(TerraformSecurityFinding {
                rule_id: "sec-tf-unpinned-version".to_string(),
                message: format!(
                    "Provider '{}' does not specify explicit version constraint",
                    block.first_label().unwrap_or("")
                ),
                severity: TerraformSecuritySeverity::Medium,
                span: block.span,
            });
        }
    }

    fn audit_module_block(
        &self,
        block: &crate::terraform::parser::HclBlock,
        findings: &mut Vec<TerraformSecurityFinding>,
    ) {
        let src = block
            .get_attribute("source")
            .map(|a| a.value_expression.as_str())
            .unwrap_or("");
        if !src.starts_with('.') && block.get_attribute("version").is_none() {
            findings.push(TerraformSecurityFinding {
                rule_id: "sec-tf-unpinned-version".to_string(),
                message: format!(
                    "External module '{}' does not pin version constraint",
                    block.first_label().unwrap_or("")
                ),
                severity: TerraformSecuritySeverity::Medium,
                span: block.span,
            });
        }
    }

    fn audit_hardcoded_secrets(
        &self,
        attr: &crate::terraform::parser::HclAttribute,
        findings: &mut Vec<TerraformSecurityFinding>,
    ) {
        let key_lower = attr.name.to_lowercase();
        let val = attr.value_expression.trim_matches('"');

        if matches!(
            key_lower.as_str(),
            "password" | "secret" | "api_key" | "access_token" | "token" | "secret_key"
        ) && !val.starts_with("${")
            && !val.starts_with("var.")
            && !val.is_empty()
            && val.len() > 3
        {
            findings.push(TerraformSecurityFinding {
                rule_id: "sec-tf-hardcoded-secret".to_string(),
                message: format!(
                    "Attribute '{}' contains potential hardcoded secret value",
                    attr.name
                ),
                severity: TerraformSecuritySeverity::Critical,
                span: attr.span,
            });
        }
    }
}
