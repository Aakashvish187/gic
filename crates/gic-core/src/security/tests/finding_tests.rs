//! Unit tests for `SecurityFinding` construction, ID stability, and field correctness.

use crate::diagnostics::position::DiagnosticPosition;
use crate::diagnostics::range::DiagnosticRange;
use crate::security::category::SecurityCategory;
use crate::security::evidence::FindingEvidence;
use crate::security::findings::SecurityFinding;
use crate::security::severity::SecuritySeverity;
use std::path::PathBuf;

fn make_range(line: usize, col: usize) -> DiagnosticRange {
    let start = DiagnosticPosition::new(line, col, 0);
    let end = DiagnosticPosition::new(line, col + 10, 10);
    DiagnosticRange::new(start, end)
}

fn make_evidence(rule_id: &str, engine: &str) -> FindingEvidence {
    FindingEvidence {
        file_path: Some(PathBuf::from("/infra/main.tf")),
        range: make_range(5, 1),
        snippet: "secret_key = \"AKIAIOSFODNN7EXAMPLE\"".to_string(),
        rule_id: rule_id.to_string(),
        source_engine: engine.to_string(),
    }
}

#[test]
fn test_finding_construction_fields() {
    let evidence = make_evidence("SEC001", "SecretScanner");
    let finding = SecurityFinding::new(
        SecuritySeverity::Critical,
        SecurityCategory::Secrets,
        "AWS Access Key Exposed",
        "An AWS access key was detected in plaintext in a Terraform file.",
        evidence,
        "Remove the key and rotate credentials immediately.",
    );

    assert_eq!(finding.severity, SecuritySeverity::Critical);
    assert_eq!(finding.category, SecurityCategory::Secrets);
    assert_eq!(finding.title, "AWS Access Key Exposed");
    assert!(!finding.description.is_empty());
    assert!(!finding.remediation.is_empty());
    assert!(!finding.id.is_empty());
    assert!(finding.timestamp > 0);
}

#[test]
fn test_finding_id_is_deterministic_for_same_inputs() {
    let ev1 = make_evidence("SEC001", "SecretScanner");
    let ev2 = make_evidence("SEC001", "SecretScanner");

    let f1 = SecurityFinding::new(
        SecuritySeverity::Critical,
        SecurityCategory::Secrets,
        "Same Title",
        "Same description",
        ev1,
        "Fix it",
    );
    let f2 = SecurityFinding::new(
        SecuritySeverity::Critical,
        SecurityCategory::Secrets,
        "Same Title",
        "Same description",
        ev2,
        "Fix it",
    );

    assert_eq!(f1.id, f2.id, "IDs must be deterministic for identical inputs");
}

#[test]
fn test_finding_id_differs_for_different_rules() {
    let ev1 = make_evidence("SEC001", "SecretScanner");
    let ev2 = make_evidence("SEC002", "SecretScanner");

    let f1 = SecurityFinding::new(
        SecuritySeverity::High,
        SecurityCategory::Credentials,
        "Title A",
        "desc",
        ev1,
        "fix",
    );
    let f2 = SecurityFinding::new(
        SecuritySeverity::High,
        SecurityCategory::Credentials,
        "Title A",
        "desc",
        ev2,
        "fix",
    );

    assert_ne!(f1.id, f2.id, "Different rule IDs must produce different finding IDs");
}

#[test]
fn test_severity_ordering() {
    assert!(SecuritySeverity::Critical > SecuritySeverity::High);
    assert!(SecuritySeverity::High > SecuritySeverity::Medium);
    assert!(SecuritySeverity::Medium > SecuritySeverity::Low);
    assert!(SecuritySeverity::Low > SecuritySeverity::Information);
}

#[test]
fn test_severity_risk_weights() {
    assert_eq!(SecuritySeverity::Information.risk_weight(), 1);
    assert_eq!(SecuritySeverity::Low.risk_weight(), 2);
    assert_eq!(SecuritySeverity::Medium.risk_weight(), 4);
    assert_eq!(SecuritySeverity::High.risk_weight(), 7);
    assert_eq!(SecuritySeverity::Critical.risk_weight(), 10);
}

#[test]
fn test_category_display_names_are_non_empty() {
    let cats = [
        SecurityCategory::Secrets,
        SecurityCategory::Credentials,
        SecurityCategory::Networking,
        SecurityCategory::Containers,
        SecurityCategory::Kubernetes,
        SecurityCategory::Terraform,
        SecurityCategory::Linux,
        SecurityCategory::Certificates,
        SecurityCategory::Compliance,
        SecurityCategory::Configuration,
    ];
    for cat in &cats {
        assert!(!cat.display_name().is_empty(), "Category {cat:?} has empty display name");
    }
}
