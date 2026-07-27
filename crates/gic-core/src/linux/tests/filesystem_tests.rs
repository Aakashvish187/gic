//! Unit tests for Linux filesystem parsers.

use crate::linux::validator::LinuxValidator;

#[test]
fn test_fstab_validation() {
    let source = "# /etc/fstab\n/dev/sda1  /boot  ext4"; // missing options/dump/pass
    let validator = LinuxValidator::new();
    let diags = validator.validate_fstab(source);

    assert!(diags.iter().any(|d| d.rule_id == "lin-fs-fstab"));
}
