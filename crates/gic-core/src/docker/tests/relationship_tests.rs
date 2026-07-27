//! Unit tests for Docker Compose Cross-Resource Relationship Validation.

use crate::docker::engine::{DockerEngine, DockerEngineOptions};

#[test]
fn test_dangling_depends_on_networks_and_volumes() {
    let source = r#"
version: '3.8'
services:
  web:
    image: nginx:1.25
    depends_on:
      - missing_db
    networks:
      - missing_net
    volumes:
      - missing_vol:/data
"#;

    let engine = DockerEngine::new(DockerEngineOptions {
        enable_cache: false,
        cache_capacity: 0,
    });
    let (diags, _) = engine.validate_compose(source);

    assert!(diags
        .iter()
        .any(|d| d.rule_id == "rel-compose-dangling-depends-on"));
    assert!(diags
        .iter()
        .any(|d| d.rule_id == "rel-compose-dangling-network"));
    assert!(diags
        .iter()
        .any(|d| d.rule_id == "rel-compose-dangling-volume"));
}
