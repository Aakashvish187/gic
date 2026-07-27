//! Unit tests for Terraform IaC security auditing rules.

use crate::terraform::engine::{TerraformEngine, TerraformEngineOptions};

#[test]
fn test_security_audit_detects_public_s3_and_hardcoded_secrets() {
    let source = r#"
resource "aws_s3_bucket" "public_bucket" {
  bucket = "my-secret-data"
  acl    = "public-read"
}

resource "aws_db_instance" "default" {
  allocated_storage   = 10
  engine              = "mysql"
  username            = "admin"
  password            = "supersecret123"
  publicly_accessible = true
}
"#;

    let engine = TerraformEngine::new(TerraformEngineOptions {
        enable_cache: false,
        cache_capacity: 0,
        ..Default::default()
    });
    let (diags, _) = engine.validate_source(source);

    assert!(diags.iter().any(|d| d.rule_id == "sec-tf-public-s3"));
    assert!(diags.iter().any(|d| d.rule_id == "sec-tf-hardcoded-secret"));
    assert!(diags.iter().any(|d| d.rule_id == "sec-tf-open-db"));
}
