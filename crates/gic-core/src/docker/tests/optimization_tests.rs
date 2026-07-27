//! Unit tests for Layer Optimization and Cache Efficiency.

use crate::docker::engine::DockerEngine;

#[test]
fn test_optimization_detects_unchained_apt_get_update() {
    let source = r#"
FROM debian:12-slim
RUN apt-get update
RUN apt-get install -y curl
USER 10001
CMD ["curl"]
"#;

    let engine = DockerEngine::default();
    let (diags, _) = engine.validate_dockerfile(source);

    assert!(diags
        .iter()
        .any(|d| d.rule_id == "opt-docker-chain-apt-update-install"));
}
