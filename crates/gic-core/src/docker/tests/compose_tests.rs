//! Unit tests for Docker Compose parsing and service validation.

use crate::docker::engine::{DockerEngine, DockerEngineOptions};

#[test]
fn test_compose_service_validation() {
    let source = r#"
version: '3.8'
services:
  web:
    image: nginx:1.25
    ports:
      - "80:80"
    restart: always
  broken_service:
    restart: invalid_policy
"#;

    let engine = DockerEngine::new(DockerEngineOptions {
        enable_cache: false,
        cache_capacity: 0,
    });
    let (diags, _) = engine.validate_compose(source);

    assert!(diags
        .iter()
        .any(|d| d.rule_id == "compose-service-missing-image-or-build"));
    assert!(diags
        .iter()
        .any(|d| d.rule_id == "compose-invalid-restart-policy"));
}
