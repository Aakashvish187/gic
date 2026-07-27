//! Unit tests for Cross-Resource Relationship Validation (Ingress -> Service, Service -> Pod).

use crate::kubernetes::engine::K8sEngine;

#[test]
fn test_dangling_ingress_backend_service() {
    let source = r#"
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: web-ingress
  namespace: default
spec:
  rules:
    - http:
        paths:
          - path: /
            backend:
              service:
                name: missing-service
                port:
                  number: 80
"#;

    let engine = K8sEngine::default();
    let (diags, _) = engine.validate(source);

    let dangling_diag = diags
        .iter()
        .find(|d| d.rule_id == "rel-k8s-dangling-ingress");
    assert!(dangling_diag.is_some());
}
