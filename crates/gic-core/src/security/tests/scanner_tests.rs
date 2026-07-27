//! Unit tests for `SecurityScanner` content scanning.

use crate::security::scanner::SecurityScanner;
use std::path::Path;

fn scanner() -> SecurityScanner {
    SecurityScanner::new().expect("SecurityScanner must initialize successfully")
}

#[test]
fn test_scanner_clean_content_returns_no_findings() {
    let s = scanner();
    let findings = s.scan_content(Some(Path::new("config.yaml")), "replicas: 3\nimage: nginx:1.25\n");
    assert!(findings.is_empty(), "Clean content must not produce security findings");
}

#[test]
fn test_scanner_detects_aws_access_key() {
    let s = scanner();
    let content = "aws_access_key_id: AKIAIOSFODNN7EXAMPLE\n";
    let findings = s.scan_content(Some(Path::new("creds.yaml")), content);
    assert!(
        !findings.is_empty(),
        "AWS access key in content must produce at least one finding"
    );
}

#[test]
fn test_scanner_detects_generic_secret_assignment() {
    let s = scanner();
    let content = "SECRET_KEY=supersecretpassword12345\n";
    let findings = s.scan_content(Some(Path::new(".env")), content);
    assert!(
        !findings.is_empty(),
        "Secret key assignment must be flagged"
    );
}

#[test]
fn test_scanner_handles_binary_like_large_content_gracefully() {
    let s = scanner();
    // 2MB of 'x' chars — scanner must skip without panicking
    let large = "x".repeat(2_097_152);
    let findings = s.scan_content(Some(Path::new("large_file.bin")), &large);
    assert!(findings.is_empty(), "Content > 1MB must be skipped and return empty findings");
}

#[test]
fn test_scanner_without_file_path() {
    let s = scanner();
    let content = "password = \"hunter2\"\n";
    // Must not panic when file_path is None
    let findings = s.scan_content(None, content);
    // findings may or may not be empty depending on scanner rules; must not panic
    let _ = findings;
}
