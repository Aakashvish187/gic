//! Unit tests for Kubernetes Resource Detector.

use crate::kubernetes::resource_detector::{K8sResourceDetector, K8sResourceKind};
use crate::yaml::parser::YamlParser;

#[test]
fn test_detect_multiple_resources() {
    let source = r#"
apiVersion: apps/v1
kind: Deployment
metadata:
  name: web-deploy
  namespace: prod
---
apiVersion: v1
kind: Service
metadata:
  name: web-svc
  namespace: prod
"#;

    let mut parser = YamlParser::new();
    let ast = parser.parse(source).unwrap();
    let mut detector = K8sResourceDetector::new();
    let resources = detector.detect_resources(&ast);

    assert_eq!(resources.len(), 2);
    assert_eq!(resources[0].kind, K8sResourceKind::Deployment);
    assert_eq!(resources[0].metadata.name, "web-deploy");
    assert_eq!(resources[0].metadata.namespace, "prod");

    assert_eq!(resources[1].kind, K8sResourceKind::Service);
    assert_eq!(resources[1].metadata.name, "web-svc");
    assert_eq!(resources[1].metadata.namespace, "prod");
}
