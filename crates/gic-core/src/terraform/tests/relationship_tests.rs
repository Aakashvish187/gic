//! Unit tests for Terraform dependency graph and circular dependency detection.

use crate::terraform::dependencies::DependencyAnalyzer;
use crate::terraform::resources::{ResourceMode, TerraformResource};

#[test]
fn test_detect_circular_dependency() {
    let res_a = TerraformResource {
        mode: ResourceMode::Managed,
        resource_type: "aws_instance".to_string(),
        name: "a".to_string(),
        address: "aws_instance.a".to_string(),
        depends_on: vec!["aws_instance.b".to_string()],
        ..Default::default()
    };

    let res_b = TerraformResource {
        mode: ResourceMode::Managed,
        resource_type: "aws_instance".to_string(),
        name: "b".to_string(),
        address: "aws_instance.b".to_string(),
        depends_on: vec!["aws_instance.a".to_string()],
        ..Default::default()
    };

    let analyzer = DependencyAnalyzer::new();
    let graph = analyzer.build_graph(&[res_a, res_b]);
    let cycles = analyzer.find_cycles(&graph);

    assert!(!cycles.is_empty());
}
