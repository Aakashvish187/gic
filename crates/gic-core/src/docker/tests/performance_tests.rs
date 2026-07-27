//! Performance and Caching Tests for Docker Intelligence Engine.

use crate::docker::engine::DockerEngine;

#[test]
fn test_large_dockerfile_validation_and_caching() {
    let mut dockerfile = String::from("FROM alpine:3.19 AS base\nWORKDIR /app\nUSER 10001\n");
    for i in 0..50 {
        dockerfile.push_str(&format!("ENV VAR_{i}=val_{i}\n"));
    }

    let engine = DockerEngine::default();

    // 1st run (Cache Miss)
    let (diags1, _) = engine.validate_dockerfile(&dockerfile);

    // 2nd run (Cache Hit)
    let (diags2, _) = engine.validate_dockerfile(&dockerfile);

    assert_eq!(diags1, diags2);
    let metrics = engine.cache().metrics();
    assert_eq!(metrics.hits, 1);
    assert_eq!(metrics.entries_count, 1);
}
