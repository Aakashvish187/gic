//! Unit tests for `SecurityAnalyzer` domain adapter aggregation.

use crate::diagnostics::position::DiagnosticPosition;
use crate::diagnostics::range::DiagnosticRange;
use crate::diagnostics::{Diagnostic, DiagnosticLevel};
use crate::parser::LanguageId;
use crate::security::analyzer::{EngineDiagnosticSource, SecurityAnalyzer};

fn make_diagnostic(rule: &str, level: DiagnosticLevel, msg: &str) -> Diagnostic {
    let start = DiagnosticPosition::new(1, 1, 0);
    let end = DiagnosticPosition::new(1, 20, 19);
    let range = DiagnosticRange::new(start, end);
    Diagnostic::new(level, msg, range, rule, LanguageId::PlainText)
}

#[test]
fn test_empty_sources_return_no_findings() {
    let analyzer = SecurityAnalyzer::new();
    let findings = analyzer.analyze_all(vec![]);
    assert!(findings.is_empty());
}

#[test]
fn test_docker_source_converts_privileged_rule() {
    let analyzer = SecurityAnalyzer::new();
    let diag = make_diagnostic(
        "DOCK001",
        DiagnosticLevel::Security,
        "Privileged container detected",
    );
    let findings = analyzer.analyze_source(EngineDiagnosticSource::Docker(&[diag]));
    assert!(
        !findings.is_empty(),
        "DOCK* rules must be converted to security findings"
    );
}

#[test]
fn test_kubernetes_source_converts_k8s_rule() {
    let analyzer = SecurityAnalyzer::new();
    let diag = make_diagnostic(
        "K8S002",
        DiagnosticLevel::Security,
        "Host PID namespace enabled",
    );
    let findings = analyzer.analyze_source(EngineDiagnosticSource::Kubernetes(&[diag]));
    assert!(
        !findings.is_empty(),
        "K8S* rules must be converted to security findings"
    );
}

#[test]
fn test_terraform_source_converts_tf_rule() {
    let analyzer = SecurityAnalyzer::new();
    let diag = make_diagnostic(
        "TF001",
        DiagnosticLevel::Security,
        "S3 bucket public ACL enabled",
    );
    let findings = analyzer.analyze_source(EngineDiagnosticSource::Terraform(&[diag]));
    assert!(
        !findings.is_empty(),
        "TF* rules must be converted to security findings"
    );
}

#[test]
fn test_git_source_converts_git_rule() {
    let analyzer = SecurityAnalyzer::new();
    let diag = make_diagnostic(
        "GIT001",
        DiagnosticLevel::Warning,
        "Force push to protected branch",
    );
    let findings = analyzer.analyze_source(EngineDiagnosticSource::Git(&[diag]));
    assert!(
        !findings.is_empty(),
        "GIT* rules must be converted to security findings"
    );
}

#[test]
fn test_analyze_all_aggregates_multiple_sources() {
    let analyzer = SecurityAnalyzer::new();
    let docker_diag = make_diagnostic("DOCK001", DiagnosticLevel::Security, "Privileged container");
    let k8s_diag = make_diagnostic("K8S001", DiagnosticLevel::Security, "No resource limits");
    let tf_diag = make_diagnostic("TF002", DiagnosticLevel::Error, "Open security group");

    let findings = analyzer.analyze_all(vec![
        EngineDiagnosticSource::Docker(&[docker_diag]),
        EngineDiagnosticSource::Kubernetes(&[k8s_diag]),
        EngineDiagnosticSource::Terraform(&[tf_diag]),
    ]);

    assert!(
        findings.len() >= 3,
        "All three engine sources must contribute findings"
    );
}

#[test]
fn test_non_security_rules_are_excluded() {
    let analyzer = SecurityAnalyzer::new();
    // Rule with prefix that does not match any adapter filter
    let diag = make_diagnostic("LINT001", DiagnosticLevel::Warning, "Trailing whitespace");
    let findings = analyzer.analyze_source(EngineDiagnosticSource::Docker(&[diag]));
    // Docker adapter only passes DOCK* rules — LINT* should produce no findings
    assert!(
        findings.is_empty(),
        "Non-domain rules must not produce findings"
    );
}
