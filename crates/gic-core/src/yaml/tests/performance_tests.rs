//! Performance and Caching Tests for YAML Intelligence Engine.

use crate::yaml::engine::YamlEngine;

#[test]
fn test_large_yaml_document_performance_and_caching() {
    let mut large_yaml = String::new();
    large_yaml.push_str("metadata:\n  name: large-cluster-config\n  namespace: production\nspec:\n  replicas: 100\n  containers:\n");
    for i in 0..500 {
        large_yaml.push_str(&format!(
            "    - name: container-{}\n      image: nginx:latest\n      port: {}\n",
            i,
            8000 + i
        ));
    }

    let engine = YamlEngine::default();

    // First validation (cache miss)
    let (diags1, _) = engine.validate(&large_yaml);

    // Second validation (cache hit)
    let (diags2, _) = engine.validate(&large_yaml);

    assert_eq!(diags1, diags2);
    let metrics = engine.cache().metrics();
    assert_eq!(metrics.hits, 1);
    assert_eq!(metrics.entries_count, 1);
}
