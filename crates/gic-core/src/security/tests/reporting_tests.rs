//! Unit tests for `SecurityReporter` risk score calculation and report structure.

use crate::diagnostics::position::DiagnosticPosition;
use crate::diagnostics::range::DiagnosticRange;
use crate::security::category::SecurityCategory;
use crate::security::evidence::FindingEvidence;
use crate::security::findings::SecurityFinding;
use crate::security::reporting::SecurityReporter;
use crate::security::severity::SecuritySeverity;

fn make_finding(severity: SecuritySeverity, category: SecurityCategory) -> SecurityFinding {
    let start = DiagnosticPosition::new(1, 1, 0);
    let end = DiagnosticPosition::new(1, 10, 9);
    let range = DiagnosticRange::new(start, end);

    SecurityFinding::new(
        severity,
        category,
        "Test Finding",
        "A test security finding.",
        FindingEvidence {
            file_path: None,
            range,
            snippet: "test snippet".to_string(),
            rule_id: "TEST001".to_string(),
            source_engine: "TestEngine".to_string(),
        },
        "Fix the issue.",
    )
}

#[test]
fn test_empty_findings_produces_zero_risk_score() {
    let reporter = SecurityReporter::new();
    let report = reporter.build_report(vec![]);
    assert_eq!(report.risk_score.value(), 0);
    assert_eq!(report.severity_counts.total(), 0);
    assert!(report.findings.is_empty());
}

#[test]
fn test_single_critical_finding_produces_high_risk_score() {
    let reporter = SecurityReporter::new();
    let findings = vec![make_finding(SecuritySeverity::Critical, SecurityCategory::Secrets)];
    let report = reporter.build_report(findings);
    // 1 Critical (weight 10) / max_weight 10 => 100%
    assert_eq!(report.risk_score.value(), 100);
    assert_eq!(report.severity_counts.critical, 1);
    assert_eq!(report.severity_counts.total(), 1);
}

#[test]
fn test_severity_counts_are_accurate() {
    let reporter = SecurityReporter::new();
    let findings = vec![
        make_finding(SecuritySeverity::Critical, SecurityCategory::Secrets),
        make_finding(SecuritySeverity::High, SecurityCategory::Credentials),
        make_finding(SecuritySeverity::High, SecurityCategory::Networking),
        make_finding(SecuritySeverity::Medium, SecurityCategory::Configuration),
        make_finding(SecuritySeverity::Low, SecurityCategory::BestPractices),
        make_finding(SecuritySeverity::Information, SecurityCategory::Compliance),
    ];
    let report = reporter.build_report(findings);
    assert_eq!(report.severity_counts.critical, 1);
    assert_eq!(report.severity_counts.high, 2);
    assert_eq!(report.severity_counts.medium, 1);
    assert_eq!(report.severity_counts.low, 1);
    assert_eq!(report.severity_counts.information, 1);
    assert_eq!(report.severity_counts.total(), 6);
}

#[test]
fn test_category_counts_are_accurate() {
    let reporter = SecurityReporter::new();
    let findings = vec![
        make_finding(SecuritySeverity::High, SecurityCategory::Secrets),
        make_finding(SecuritySeverity::High, SecurityCategory::Secrets),
        make_finding(SecuritySeverity::Medium, SecurityCategory::Networking),
    ];
    let report = reporter.build_report(findings);
    let secrets_key = SecurityCategory::Secrets.display_name().to_string();
    let net_key = SecurityCategory::Networking.display_name().to_string();
    assert_eq!(report.category_counts.get(&secrets_key).copied().unwrap_or(0), 2);
    assert_eq!(report.category_counts.get(&net_key).copied().unwrap_or(0), 1);
}

#[test]
fn test_risk_score_label_mapping() {
    use crate::security::reporting::RiskScore;
    assert_eq!(RiskScore(0).label(), "Minimal");
    assert_eq!(RiskScore(25).label(), "Low");
    assert_eq!(RiskScore(45).label(), "Medium");
    assert_eq!(RiskScore(65).label(), "High");
    assert_eq!(RiskScore(90).label(), "Critical");
}

#[test]
fn test_report_timestamp_is_set() {
    let reporter = SecurityReporter::new();
    let report = reporter.build_report(vec![]);
    assert!(report.generated_at_ms > 0, "Report timestamp must be set");
}
