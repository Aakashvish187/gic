//! Docker Production Best Practices Recommendations Engine.

use crate::docker::dockerfile::DockerfileAST;
use crate::docker::instructions::InstructionKind;

/// Best practice recommendation item.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BestPracticeRecommendation {
    pub rule_id: String,
    pub message: String,
    pub line: usize,
}

/// Best practice rules evaluator.
#[derive(Debug, Clone, Default)]
pub struct DockerBestPracticesAnalyzer;

impl DockerBestPracticesAnalyzer {
    pub fn new() -> Self {
        Self
    }

    pub fn evaluate_dockerfile(&self, ast: &DockerfileAST) -> Vec<BestPracticeRecommendation> {
        let mut recommendations = Vec::new();
        let mut has_healthcheck = false;

        for inst in &ast.instructions {
            match inst.kind {
                InstructionKind::From { ref image, .. } => {
                    if image.contains("ubuntu") || image.contains("debian") {
                        recommendations.push(BestPracticeRecommendation {
                            rule_id: "bp-docker-small-base-image".to_string(),
                            message: format!("Consider using a smaller base image (e.g. Alpine or Distroless) instead of '{image}'"),
                            line: inst.line,
                        });
                    }
                }
                InstructionKind::Healthcheck { .. } => {
                    has_healthcheck = true;
                }
                _ => {}
            }
        }

        if !has_healthcheck && !ast.instructions.is_empty() {
            if let Some(last) = ast.instructions.last() {
                recommendations.push(BestPracticeRecommendation {
                    rule_id: "bp-docker-missing-healthcheck".to_string(),
                    message: "Consider adding an explicit 'HEALTHCHECK' instruction to monitor container health".to_string(),
                    line: last.line,
                });
            }
        }

        recommendations
    }
}
