//! Unit tests for SSH validation.

use crate::linux::validator::LinuxValidator;

#[test]
fn test_sshd_dangerous_configs() {
    let source = "Port 22\nPermitRootLogin yes\nPasswordAuthentication yes";
    let validator = LinuxValidator::new();
    let diags = validator.validate_sshd(source);

    assert!(diags.iter().any(|d| d.rule_id == "sec-ssh-root-login"));
    assert!(diags.iter().any(|d| d.rule_id == "sec-ssh-password-auth"));
}
