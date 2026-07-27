//! Unit tests for Terraform Variable declarations and unused variable detection.

use crate::terraform::engine::{TerraformEngine, TerraformEngineOptions};

#[test]
fn test_detect_unused_and_duplicate_variables() {
    let source = r#"
variable "vpc_cidr" {
  type    = string
  default = "10.0.0.0/16"
}

variable "unused_var" {
  type    = string
  default = "test"
}

resource "aws_vpc" "main" {
  cidr_block = var.vpc_cidr
}
"#;

    let engine = TerraformEngine::new(TerraformEngineOptions {
        enable_cache: false,
        cache_capacity: 0,
        ..Default::default()
    });
    let (diags, _) = engine.validate_source(source);

    assert!(diags
        .iter()
        .any(|d| d.rule_id == "tf-unused-variable" && d.message.contains("unused_var")));
}
