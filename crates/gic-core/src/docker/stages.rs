//! Multi-Stage Build Graph Analyzer for Dockerfiles.
//!
//! Tracks build stage definitions (`FROM image AS stage_name`) and validates `--from=<stage>`
//! cross-stage references in `COPY` instructions.

use std::collections::HashMap;

use crate::docker::dockerfile::DockerfileAST;
use crate::docker::instructions::InstructionKind;

/// Individual stage in a multi-stage Dockerfile build.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildStage {
    /// Zero-based stage index.
    pub index: usize,
    /// Base image string.
    pub image: String,
    /// Optional stage name alias (`AS stage_name`).
    pub alias: Option<String>,
    /// Line number where `FROM` occurs.
    pub line: usize,
    /// Number of instructions within this stage.
    pub instruction_count: usize,
}

/// Diagnostic issue in multi-stage build references.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StageReferenceIssue {
    /// Target referenced stage name/index that was missing.
    pub referenced_stage: String,
    /// Line number of the `COPY --from=` instruction.
    pub line: usize,
    /// Diagnostic error message.
    pub message: String,
}

/// Multi-stage build analyzer.
#[derive(Debug, Clone, Default)]
pub struct MultiStageAnalyzer;

impl MultiStageAnalyzer {
    /// Creates a new MultiStageAnalyzer.
    pub fn new() -> Self {
        Self
    }

    /// Extracts all build stages from a `DockerfileAST`.
    pub fn extract_stages(&self, ast: &DockerfileAST) -> Vec<BuildStage> {
        let mut stages = Vec::new();
        let mut current_stage: Option<BuildStage> = None;

        for inst in &ast.instructions {
            if let InstructionKind::From {
                ref image,
                ref stage_alias,
                ..
            } = inst.kind
            {
                if let Some(st) = current_stage.take() {
                    stages.push(st);
                }
                let idx = stages.len();
                current_stage = Some(BuildStage {
                    index: idx,
                    image: image.clone(),
                    alias: stage_alias.clone(),
                    line: inst.line,
                    instruction_count: 1,
                });
            } else if let Some(ref mut st) = current_stage {
                st.instruction_count += 1;
            }
        }

        if let Some(st) = current_stage {
            stages.push(st);
        }

        stages
    }

    /// Validates cross-stage `--from=<stage>` references in `COPY` instructions.
    pub fn validate_stage_references(&self, ast: &DockerfileAST) -> Vec<StageReferenceIssue> {
        let stages = self.extract_stages(ast);
        let mut known_stages: HashMap<String, usize> = HashMap::new();

        for st in &stages {
            known_stages.insert(st.index.to_string(), st.index);
            if let Some(ref alias) = st.alias {
                known_stages.insert(alias.to_string(), st.index);
            }
        }

        let mut issues = Vec::new();
        let mut current_stage_idx = 0;

        for inst in &ast.instructions {
            if let InstructionKind::From { .. } = inst.kind {
                current_stage_idx = known_stages
                    .get(&inst.line.to_string())
                    .copied()
                    .unwrap_or(current_stage_idx);
            }

            if let InstructionKind::Copy {
                from_stage: Some(ref target),
                ..
            } = inst.kind
            {
                if let Some(&target_idx) = known_stages.get(target) {
                    if target_idx >= current_stage_idx {
                        issues.push(StageReferenceIssue {
                            referenced_stage: target.clone(),
                            line: inst.line,
                            message: format!(
                                "COPY --from='{target}' references stage defined at or after current stage"
                            ),
                        });
                    }
                } else {
                    issues.push(StageReferenceIssue {
                        referenced_stage: target.clone(),
                        line: inst.line,
                        message: format!(
                            "COPY --from='{target}' references non-existent build stage"
                        ),
                    });
                }
            }
        }

        issues
    }
}
