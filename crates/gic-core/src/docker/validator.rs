//! Central Docker & Compose Validation Engine.
//!
//! Coordinates Dockerfile instruction analysis, Compose YAML validation, security audits,
//! production best practices, layer optimization, and cross-resource relationship graph validation.

use crate::docker::best_practices::DockerBestPracticesAnalyzer;
use crate::docker::compose::ComposeDetector;
use crate::docker::compose_networks::ComposeNetworkValidator;
use crate::docker::compose_services::ComposeServiceValidator;
use crate::docker::compose_volumes::ComposeVolumeValidator;
use crate::docker::dockerfile::DockerfileParser;
use crate::docker::instructions::InstructionKind;
use crate::docker::optimization::DockerOptimizationAnalyzer;
use crate::docker::security::{DockerSecurityAnalyzer, DockerSecuritySeverity};
use crate::docker::stages::MultiStageAnalyzer;
use crate::yaml::parser::{Position, Span, YamlAST, YamlNode, YamlValue};

/// Diagnostic severity for Docker validation findings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DockerSeverity {
    Error,
    Warning,
    Info,
    Hint,
}

/// Primary diagnostic item produced by `DockerValidator`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DockerDiagnostic {
    /// Rule identifier.
    pub rule_id: String,
    /// Human-readable message.
    pub message: String,
    /// Severity level.
    pub severity: DockerSeverity,
    /// Target span location in source code.
    pub span: Span,
    /// Quick-fix proposal.
    pub quick_fix: Option<(String, String)>,
}

/// Central Docker and Docker Compose validator.
#[derive(Debug, Clone, Default)]
pub struct DockerValidator {
    dockerfile_parser: DockerfileParser,
    stage_analyzer: MultiStageAnalyzer,
    compose_detector: ComposeDetector,
    service_validator: ComposeServiceValidator,
    network_validator: ComposeNetworkValidator,
    volume_validator: ComposeVolumeValidator,
    security_analyzer: DockerSecurityAnalyzer,
    best_practices_analyzer: DockerBestPracticesAnalyzer,
    optimization_analyzer: DockerOptimizationAnalyzer,
}

impl DockerValidator {
    /// Creates a new DockerValidator.
    pub fn new() -> Self {
        Self::default()
    }

    /// Validates raw Dockerfile source code.
    pub fn validate_dockerfile(&self, source: &str) -> Vec<DockerDiagnostic> {
        let mut diagnostics = Vec::new();
        let ast = match self.dockerfile_parser.parse(source) {
            Ok(ast) => ast,
            Err(err) => {
                let empty_pos = Position::new(1, 1, 0);
                diagnostics.push(DockerDiagnostic {
                    rule_id: "dockerfile-syntax-error".to_string(),
                    message: err.to_string(),
                    severity: DockerSeverity::Error,
                    span: Span::new(empty_pos, empty_pos),
                    quick_fix: None,
                });
                return diagnostics;
            }
        };

        // 1. Check multi-stage build stage references
        for issue in self.stage_analyzer.validate_stage_references(&ast) {
            let start_pos = Position::new(issue.line, 1, 0);
            diagnostics.push(DockerDiagnostic {
                rule_id: "docker-invalid-stage-reference".to_string(),
                message: issue.message,
                severity: DockerSeverity::Error,
                span: Span::new(start_pos, start_pos),
                quick_fix: None,
            });
        }

        // 2. Check CMD / ENTRYPOINT missing or duplicate
        let mut cmd_count = 0;
        for inst in &ast.instructions {
            if matches!(inst.kind, InstructionKind::Cmd { .. }) {
                cmd_count += 1;
                if cmd_count > 1 {
                    diagnostics.push(DockerDiagnostic {
                        rule_id: "dockerfile-multiple-cmd".to_string(),
                        message: "Multiple 'CMD' instructions found; only the last 'CMD' will take effect".to_string(),
                        severity: DockerSeverity::Warning,
                        span: inst.span,
                        quick_fix: None,
                    });
                }
            }
        }

        // 3. Security Audit
        for finding in self.security_analyzer.audit_dockerfile(&ast) {
            let severity = match finding.severity {
                DockerSecuritySeverity::Critical | DockerSecuritySeverity::High => {
                    DockerSeverity::Error
                }
                DockerSecuritySeverity::Medium => DockerSeverity::Warning,
                DockerSecuritySeverity::Low => DockerSeverity::Info,
            };
            let quick_fix = finding
                .fix_suggestion
                .map(|fix| ("Apply security fix".to_string(), fix));
            diagnostics.push(DockerDiagnostic {
                rule_id: finding.rule_id,
                message: finding.message,
                severity,
                span: finding.span,
                quick_fix,
            });
        }

        // 4. Production Best Practices
        for rec in self.best_practices_analyzer.evaluate_dockerfile(&ast) {
            let start_pos = Position::new(rec.line, 1, 0);
            diagnostics.push(DockerDiagnostic {
                rule_id: rec.rule_id,
                message: rec.message,
                severity: DockerSeverity::Hint,
                span: Span::new(start_pos, start_pos),
                quick_fix: None,
            });
        }

        // 5. Optimization Analysis
        for opt in self.optimization_analyzer.analyze(&ast) {
            let start_pos = Position::new(opt.line, 1, 0);
            diagnostics.push(DockerDiagnostic {
                rule_id: opt.rule_id,
                message: opt.message,
                severity: DockerSeverity::Info,
                span: Span::new(start_pos, start_pos),
                quick_fix: None,
            });
        }

        diagnostics
    }

    /// Validates a parsed `YamlAST` containing a Docker Compose document.
    pub fn validate_compose_ast(&self, ast: &YamlAST) -> Vec<DockerDiagnostic> {
        let mut diagnostics = Vec::new();
        let compose_docs = self.compose_detector.parse_compose(ast);

        for doc in &compose_docs {
            // 1. Service spec validation
            for (svc_name, node) in &doc.services {
                for issue in self.service_validator.validate_service(svc_name, node) {
                    diagnostics.push(DockerDiagnostic {
                        rule_id: issue.rule_id,
                        message: issue.message,
                        severity: DockerSeverity::Error,
                        span: node.span,
                        quick_fix: None,
                    });
                }
            }

            // 2. Network spec validation
            for (net_name, node) in &doc.networks {
                for issue in self.network_validator.validate_network(net_name, node) {
                    diagnostics.push(DockerDiagnostic {
                        rule_id: issue.rule_id,
                        message: issue.message,
                        severity: DockerSeverity::Error,
                        span: node.span,
                        quick_fix: None,
                    });
                }
            }

            // 3. Volume spec validation
            for (vol_name, node) in &doc.volumes {
                for issue in self.volume_validator.validate_volume(vol_name, node) {
                    diagnostics.push(DockerDiagnostic {
                        rule_id: issue.rule_id,
                        message: issue.message,
                        severity: DockerSeverity::Error,
                        span: node.span,
                        quick_fix: None,
                    });
                }
            }

            // 4. Cross-Resource Relationship Graph Validation
            self.validate_compose_relationships(doc, &mut diagnostics);

            // 5. Security audit
            for finding in self.security_analyzer.audit_compose(doc) {
                let severity = match finding.severity {
                    DockerSecuritySeverity::Critical | DockerSecuritySeverity::High => {
                        DockerSeverity::Error
                    }
                    DockerSecuritySeverity::Medium => DockerSeverity::Warning,
                    DockerSecuritySeverity::Low => DockerSeverity::Info,
                };
                diagnostics.push(DockerDiagnostic {
                    rule_id: finding.rule_id,
                    message: finding.message,
                    severity,
                    span: finding.span,
                    quick_fix: None,
                });
            }
        }

        diagnostics
    }

    fn validate_compose_relationships(
        &self,
        doc: &crate::docker::compose::ComposeDocument,
        diagnostics: &mut Vec<DockerDiagnostic>,
    ) {
        for (svc_name, node) in &doc.services {
            if let YamlValue::Mapping(ref map) = node.value {
                for pair in &map.pairs {
                    match pair.key.value.as_str() {
                        "depends_on" => {
                            for (dep_name, span) in extract_list_or_map_keys(&pair.value) {
                                if !doc.services.contains_key(&dep_name) {
                                    diagnostics.push(DockerDiagnostic {
                                        rule_id: "rel-compose-dangling-depends-on".to_string(),
                                        message: format!("Service '{svc_name}' depends_on undefined service '{dep_name}'"),
                                        severity: DockerSeverity::Error,
                                        span,
                                        quick_fix: None,
                                    });
                                }
                            }
                        }
                        "networks" => {
                            for (net_name, span) in extract_list_or_map_keys(&pair.value) {
                                if !doc.networks.contains_key(&net_name) && net_name != "default" {
                                    diagnostics.push(DockerDiagnostic {
                                        rule_id: "rel-compose-dangling-network".to_string(),
                                        message: format!("Service '{svc_name}' references undefined top-level network '{net_name}'"),
                                        severity: DockerSeverity::Error,
                                        span,
                                        quick_fix: None,
                                    });
                                }
                            }
                        }
                        "volumes" => {
                            for (vol_str, span) in extract_list_or_map_keys(&pair.value) {
                                let named_vol = vol_str.split(':').next().unwrap_or("").trim();
                                if !named_vol.starts_with('.')
                                    && !named_vol.starts_with('/')
                                    && !named_vol.starts_with('~')
                                    && !named_vol.is_empty()
                                    && !doc.volumes.contains_key(named_vol)
                                {
                                    diagnostics.push(DockerDiagnostic {
                                            rule_id: "rel-compose-dangling-volume".to_string(),
                                            message: format!("Service '{svc_name}' references undefined top-level volume '{named_vol}'"),
                                            severity: DockerSeverity::Error,
                                            span,
                                            quick_fix: None,
                                        });
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}

fn extract_list_or_map_keys(node: &YamlNode) -> Vec<(String, Span)> {
    let mut results = Vec::new();
    match &node.value {
        YamlValue::Sequence(seq) => {
            for item in &seq.items {
                match &item.value {
                    YamlValue::Scalar(s) => results.push((s.value.clone(), item.span)),
                    YamlValue::Mapping(m) => {
                        for p in &m.pairs {
                            results.push((p.key.value.clone(), p.span));
                        }
                    }
                    _ => {}
                }
            }
        }
        YamlValue::Mapping(m) => {
            for p in &m.pairs {
                results.push((p.key.value.clone(), p.span));
            }
        }
        YamlValue::Scalar(s) => {
            results.push((s.value.clone(), node.span));
        }
        _ => {}
    }
    results
}
