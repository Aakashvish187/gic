//! Performance and benchmark unit tests for Terraform Engine.

use crate::terraform::engine::{TerraformEngine, TerraformEngineOptions};

#[test]
fn test_large_terraform_workspace_performance_and_caching() {
    let mut source = String::new();
    for i in 0..100 {
        source.push_str(&format!(
            "resource \"aws_s3_bucket\" \"bucket_{i}\" {{\n  bucket = \"my-bucket-{i}\"\n  tags = {{\n    Env = \"Dev\"\n  }}\n}}\n\n"
        ));
    }

    let engine = TerraformEngine::new(TerraformEngineOptions {
        enable_cache: true,
        cache_capacity: 100,
        ..Default::default()
    });

    let (diags1, _) = engine.validate_source(&source);
    assert_eq!(engine.cache().len(), 1);

    let (diags2, _) = engine.validate_source(&source);
    assert_eq!(diags1, diags2);
}
