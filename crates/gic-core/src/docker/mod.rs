//! Docker & Docker Compose Intelligence Engine for GIC (General Infrastructure Console).
//!
//! Provides production-grade Dockerfile parsing, instruction AST generation, multi-stage build tracking,
//! Docker Compose YAML structure detection, container security auditing (root user, unpinned tags, secrets),
//! production best practice recommendations, layer optimization, and cross-resource relationship graph validation.

#![forbid(unsafe_code)]

pub mod best_practices;
pub mod cache;
pub mod compose;
pub mod compose_networks;
pub mod compose_services;
pub mod compose_volumes;
pub mod diagnostics;
pub mod dockerfile;
pub mod engine;
pub mod errors;
pub mod image_analysis;
pub mod instructions;
pub mod optimization;
pub mod security;
pub mod stages;
pub mod validator;

#[cfg(test)]
pub mod tests;

pub use best_practices::{BestPracticeRecommendation, DockerBestPracticesAnalyzer};
pub use cache::{DockerCache, DockerCacheEntry, DockerCacheMetrics};
pub use compose::{ComposeDetector, ComposeDocument};
pub use compose_networks::{ComposeNetworkIssue, ComposeNetworkValidator};
pub use compose_services::{ComposeServiceIssue, ComposeServiceValidator};
pub use compose_volumes::{ComposeVolumeIssue, ComposeVolumeValidator};
pub use diagnostics::{convert_docker_diagnostic, convert_docker_diagnostics};
pub use dockerfile::{DockerfileAST, DockerfileParser};
pub use engine::{DockerEngine, DockerEngineOptions};
pub use errors::{DockerError, DockerResult};
pub use image_analysis::{ImageAnalyzer, ImageMetricsReport};
pub use instructions::{DockerfileInstruction, InstructionKind};
pub use optimization::{DockerOptimizationAnalyzer, OptimizationFinding};
pub use security::{DockerSecurityAnalyzer, DockerSecurityFinding, DockerSecuritySeverity};
pub use stages::{BuildStage, MultiStageAnalyzer, StageReferenceIssue};
pub use validator::{DockerDiagnostic, DockerSeverity, DockerValidator};
