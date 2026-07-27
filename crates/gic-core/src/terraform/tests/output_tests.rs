//! Unit tests for Terraform Output declarations and duplicate detection.

use crate::terraform::engine::{TerraformEngine, TerraformEngineOptions};

#[test]
fn test_detect_duplicate_outputs() {
    let source = r#"
output "vpc_id" {
  value = "vpc-12345"
}

output "vpc_id" {
  value = "vpc-67890"
}
"#;

    let engine = TerraformEngine::new(TerraformEngineOptions {
        enable_cache: false,
        cache_capacity: 0,
        ..Default::default()
    });
    let (diags, _) = engine.validate_source(source);

    assert!(diags.iter().any(|d| d.rule_id == "tf-duplicate-output"));
}
