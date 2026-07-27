//! Unit tests for Kubernetes Security Auditing.

use crate::kubernetes::engine::K8sEngine;

#[test]
fn test_security_audit_detects_privileged_and_unpinned_images() {
    let source = r#"
apiVersion: apps/v1
kind: Deployment
metadata:
  name: insecure-deploy
spec:
  template:
    spec:
      hostNetwork: true
      containers:
        - name: app
          image: nginx:latest
          privileged: true
"#;

    let engine = K8sEngine::default();
    let (diags, _) = engine.validate(source);

    assert!(diags.iter().any(|d| d.rule_id == "sec-k8s-no-host-network"));
    assert!(diags
        .iter()
        .any(|d| d.rule_id == "sec-k8s-no-latest-image-tag"));
    assert!(diags
        .iter()
        .any(|d| d.rule_id == "sec-k8s-no-privileged-containers"));
}
