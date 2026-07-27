//! Unit tests for Systemd parsing.

use crate::linux::engine::{LinuxEngine, LinuxEngineOptions};

#[test]
fn test_systemd_missing_exec_start() {
    let source = r#"
[Unit]
Description=My Service

[Service]
Restart=always

[Install]
WantedBy=multi-user.target
"#;

    let engine = LinuxEngine::new(LinuxEngineOptions::default());
    let (diags, _) = engine.validate_systemd(source);

    assert!(diags
        .iter()
        .any(|d| d.rule_id == "lin-systemd-missing-execstart"));
}
