//! Unit tests for Container Security Audits.

use crate::docker::engine::DockerEngine;

#[test]
fn test_security_audit_detects_root_user_unpinned_tag_and_secrets() {
    let source = r#"
FROM ubuntu:latest
ENV DB_PASSWORD=supersecretpassword123
ADD http://example.com/app.tar.gz /tmp/
USER root
CMD ["/tmp/app"]
"#;

    let engine = DockerEngine::default();
    let (diags, _) = engine.validate_dockerfile(source);

    assert!(diags
        .iter()
        .any(|d| d.rule_id == "sec-docker-no-latest-tag"));
    assert!(diags
        .iter()
        .any(|d| d.rule_id == "sec-docker-secret-in-env"));
    assert!(diags.iter().any(|d| d.rule_id == "sec-docker-user-root"));
}

#[test]
fn test_compose_security_audit_detects_privileged_and_host_network() {
    let source = r#"
version: '3.8'
services:
  app:
    image: myapp:1.0
    privileged: true
    network_mode: host
"#;

    let engine = DockerEngine::default();
    let (diags, _) = engine.validate_compose(source);

    assert!(diags
        .iter()
        .any(|d| d.rule_id == "sec-compose-privileged-container"));
    assert!(diags
        .iter()
        .any(|d| d.rule_id == "sec-compose-host-network"));
}
