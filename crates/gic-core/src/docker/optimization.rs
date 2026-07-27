//! Dockerfile Layer Optimization and Cache Efficiency Analyzer.

use crate::docker::dockerfile::DockerfileAST;
use crate::docker::instructions::InstructionKind;

/// Layer optimization item.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OptimizationFinding {
    pub rule_id: String,
    pub message: String,
    pub line: usize,
}

/// Dockerfile layer optimizer.
#[derive(Debug, Clone, Default)]
pub struct DockerOptimizationAnalyzer;

impl DockerOptimizationAnalyzer {
    pub fn new() -> Self {
        Self
    }

    pub fn analyze(&self, ast: &DockerfileAST) -> Vec<OptimizationFinding> {
        let mut findings = Vec::new();
        let mut run_count = 0;

        for inst in &ast.instructions {
            if let InstructionKind::Run { ref command, .. } = inst.kind {
                run_count += 1;
                let cmd_str = command.join(" ");
                if cmd_str.contains("apt-get update") && !cmd_str.contains("apt-get install") {
                    findings.push(OptimizationFinding {
                        rule_id: "opt-docker-chain-apt-update-install".to_string(),
                        message: "'RUN apt-get update' should be chained in the same layer with 'apt-get install'".to_string(),
                        line: inst.line,
                    });
                }
            }
        }

        if run_count > 8 {
            if let Some(first_run) = ast
                .instructions
                .iter()
                .find(|i| matches!(i.kind, InstructionKind::Run { .. }))
            {
                findings.push(OptimizationFinding {
                    rule_id: "opt-docker-high-layer-count".to_string(),
                    message: format!("Dockerfile contains {run_count} RUN instructions; consider chaining commands with '&&' to reduce layer count"),
                    line: first_run.line,
                });
            }
        }

        findings
    }
}
