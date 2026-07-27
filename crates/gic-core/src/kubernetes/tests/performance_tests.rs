//! Performance and Caching Tests for Kubernetes Intelligence Engine.

use crate::kubernetes::engine::K8sEngine;

#[test]
fn test_large_kubernetes_manifest_validation_and_caching() {
    let mut manifest = String::new();

    for i in 0..100 {
        manifest.push_str(&format!(
            "apiVersion: apps/v1\nkind: Deployment\nmetadata:\n  name: app-{}\n  namespace: prod\n  labels:\n    app.kubernetes.io/name: app-{}\nspec:\n  replicas: 2\n  selector:\n    matchLabels:\n      app: app-{}\n  template:\n    metadata:\n      labels:\n        app: app-{}\n    spec:\n      containers:\n        - name: web\n          image: nginx:1.25\n---\n",
            i, i, i, i
        ));
    }

    let engine = K8sEngine::default();

    // 1st execution (Cache Miss)
    let (diags1, _) = engine.validate(&manifest);

    // 2nd execution (Cache Hit)
    let (diags2, _) = engine.validate(&manifest);

    assert_eq!(diags1, diags2);
    let metrics = engine.cache().metrics();
    assert_eq!(metrics.hits, 1);
    assert_eq!(metrics.entries_count, 1);
}
