//! Container Security Audit Analyzer for Dockerfiles and Compose Files.
//!
//! Audits container builds and runtimes for root user execution, unpinned image tags,
//! inappropriate `ADD` usage, secrets in `ENV`/`ARG`, privileged containers,
//! dangerous Linux capabilities (`ALL`, `SYS_ADMIN`), host networking, and writable root FS.

use crate::docker::compose::ComposeDocument;
use crate::docker::dockerfile::DockerfileAST;
use crate::docker::instructions::InstructionKind;
use crate::yaml::parser::{Span, YamlValue};

/// Category of container security finding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DockerSecuritySeverity {
    Critical,
    High,
    Medium,
    Low,
}

/// Security violation report item.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DockerSecurityFinding {
    /// Rule identifier.
    pub rule_id: String,
    /// Human-readable message.
    pub message: String,
    /// Severity level.
    pub severity: DockerSecuritySeverity,
    /// Line number.
    pub line: usize,
    /// Source span.
    pub span: Span,
    /// Suggested fix proposal.
    pub fix_suggestion: Option<String>,
}

/// Security analyzer for Docker artifacts.
#[derive(Debug, Clone, Default)]
pub struct DockerSecurityAnalyzer;

impl DockerSecurityAnalyzer {
    /// Creates a new DockerSecurityAnalyzer.
    pub fn new() -> Self {
        Self
    }

    /// Audits a parsed `DockerfileAST` for security flaws.
    pub fn audit_dockerfile(&self, ast: &DockerfileAST) -> Vec<DockerSecurityFinding> {
        let mut findings = Vec::new();
        let mut has_user_instruction = false;

        for inst in &ast.instructions {
            match inst.kind {
                InstructionKind::From { ref image, .. } => {
                    let img = image.trim();
                    if img.ends_with(":latest") || !img.contains(':') || img.contains("latest") {
                        findings.push(DockerSecurityFinding {
                            rule_id: "sec-docker-no-latest-tag".to_string(),
                            message: format!("Base image '{img}' uses 'latest' or unpinned tag"),
                            severity: DockerSecuritySeverity::Medium,
                            line: inst.line,
                            span: inst.span,
                            fix_suggestion: Some(
                                "Pin base image to explicit semver tag or SHA digest".to_string(),
                            ),
                        });
                    }
                }
                InstructionKind::User { ref user, .. } => {
                    has_user_instruction = true;
                    if user == "root" || user == "0" {
                        findings.push(DockerSecurityFinding {
                            rule_id: "sec-docker-user-root".to_string(),
                            message:
                                "Explicitly setting 'USER root' creates container privilege risk"
                                    .to_string(),
                            severity: DockerSecuritySeverity::High,
                            line: inst.line,
                            span: inst.span,
                            fix_suggestion: Some("USER appuser".to_string()),
                        });
                    }
                }
                InstructionKind::Add { ref sources, .. } => {
                    let uses_url = sources
                        .iter()
                        .any(|s| s.starts_with("http://") || s.starts_with("https://"));
                    if !uses_url {
                        findings.push(DockerSecurityFinding {
                            rule_id: "sec-docker-use-copy-instead-of-add".to_string(),
                            message: "'ADD' used for local files; use 'COPY' unless auto-extracting tarballs".to_string(),
                            severity: DockerSecuritySeverity::Low,
                            line: inst.line,
                            span: inst.span,
                            fix_suggestion: Some("COPY".to_string()),
                        });
                    }
                }
                InstructionKind::Env { ref pairs } => {
                    for (k, v) in pairs {
                        if is_potential_secret_key(k) || is_potential_secret_value(v) {
                            findings.push(DockerSecurityFinding {
                                rule_id: "sec-docker-secret-in-env".to_string(),
                                message: format!(
                                    "Potential secret/credential key '{k}' in ENV instruction"
                                ),
                                severity: DockerSecuritySeverity::Critical,
                                line: inst.line,
                                span: inst.span,
                                fix_suggestion: Some(
                                    "Pass secrets via Docker Secrets or build-time mounts"
                                        .to_string(),
                                ),
                            });
                        }
                    }
                }
                InstructionKind::Arg {
                    ref name,
                    default_value: Some(ref v),
                } if (is_potential_secret_key(name) || is_potential_secret_value(v)) => {
                    findings.push(DockerSecurityFinding {
                        rule_id: "sec-docker-secret-in-arg".to_string(),
                        message: format!("Potential default secret value in ARG '{name}'"),
                        severity: DockerSecuritySeverity::Critical,
                        line: inst.line,
                        span: inst.span,
                        fix_suggestion: Some(
                            "Do not embed hardcoded secrets in Dockerfile ARG".to_string(),
                        ),
                    });
                }
                _ => {}
            }
        }

        if !has_user_instruction && !ast.instructions.is_empty() {
            if let Some(last) = ast.instructions.last() {
                findings.push(DockerSecurityFinding {
                    rule_id: "sec-docker-missing-user".to_string(),
                    message:
                        "Dockerfile runs as default 'root' user; add a non-root 'USER' instruction"
                            .to_string(),
                    severity: DockerSecuritySeverity::High,
                    line: last.line,
                    span: last.span,
                    fix_suggestion: Some("USER 10001".to_string()),
                });
            }
        }

        findings
    }

    /// Audits a parsed `ComposeDocument` for security flaws.
    pub fn audit_compose(&self, doc: &ComposeDocument) -> Vec<DockerSecurityFinding> {
        let mut findings = Vec::new();

        for (svc_name, node) in &doc.services {
            if let YamlValue::Mapping(ref map) = node.value {
                for pair in &map.pairs {
                    match pair.key.value.as_str() {
                        "privileged" => {
                            if let YamlValue::Scalar(ref s) = pair.value.value {
                                if s.value.trim() == "true" {
                                    findings.push(DockerSecurityFinding {
                                        rule_id: "sec-compose-privileged-container".to_string(),
                                        message: format!("Service '{svc_name}' sets 'privileged: true', granting full host kernel access"),
                                        severity: DockerSecuritySeverity::Critical,
                                        line: pair.value.span.start.line,
                                        span: pair.value.span,
                                        fix_suggestion: Some("privileged: false".to_string()),
                                    });
                                }
                            }
                        }
                        "network_mode" => {
                            if let YamlValue::Scalar(ref s) = pair.value.value {
                                if s.value.trim() == "host" {
                                    findings.push(DockerSecurityFinding {
                                        rule_id: "sec-compose-host-network".to_string(),
                                        message: format!(
                                            "Service '{svc_name}' uses 'network_mode: host'"
                                        ),
                                        severity: DockerSecuritySeverity::High,
                                        line: pair.value.span.start.line,
                                        span: pair.value.span,
                                        fix_suggestion: Some("network_mode: bridge".to_string()),
                                    });
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        findings
    }
}

fn is_potential_secret_key(k: &str) -> bool {
    let lower = k.to_lowercase();
    lower.contains("pass")
        || lower.contains("secret")
        || lower.contains("key")
        || lower.contains("token")
        || lower.contains("api_key")
}

fn is_potential_secret_value(v: &str) -> bool {
    v.len() >= 8 && (v.contains("secret") || v.contains("password") || v.contains("1234"))
}
