//! Unit tests for Kubernetes Production Best Practices.

use crate::kubernetes::engine::K8sEngine;

#[test]
fn test_best_practices_detects_default_namespace_and_missing_recommended_labels() {
    let source = r#"
apiVersion: apps/v1
kind: Deployment
metadata:
  name: unlabelled-app
spec:
  selector:
    matchLabels:
      app: unlabelled-app
  template:
    metadata:
      labels:
        app: unlabelled-app
    spec:
      containers:
        - name: web
          image: nginx:1.25
"#;

    let engine = K8sEngine::default();
    let (diags, _) = engine.validate(source);

    assert!(diags
        .iter()
        .any(|d| d.rule_id == "bp-k8s-explicit-namespace"));
    assert!(diags
        .iter()
        .any(|d| d.rule_id == "bp-k8s-recommended-labels"));
}
