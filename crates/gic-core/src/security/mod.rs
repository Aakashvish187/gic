//! # Security Intelligence Engine
//!
//! Centralized security module for GIC, aggregating findings from all infrastructure
//! intelligence engines (Docker, Kubernetes, Terraform, Linux, Networking, Git, YAML).
//!
//! ## Architecture
//!
//! ```text
//! SecurityEngine (façade)
//! ├── SecurityScanner       — secrets, credentials, certificates (content-level)
//! ├── SecurityAnalyzer      — domain adapters (Docker, K8s, Terraform, Linux, Net, Git, YAML)
//! ├── PolicyRegistry        — compliance policies and evaluators
//! ├── SecurityReporter      — risk scoring and report generation
//! ├── SecurityReportFormatter — JSON, Markdown, PlainText output
//! ├── SecurityDiagnostics   — GIC Diagnostic adapter (editor UI integration)
//! ├── SecurityCache         — thread-safe findings and report cache
//! └── SecurityMetrics       — operational telemetry
//! ```

pub mod analyzer;
pub mod cache;
pub mod category;
pub mod certificates;
pub mod compliance;
pub mod credentials;
pub mod diagnostics;
pub mod docker;
pub mod engine;
pub mod errors;
pub mod evidence;
pub mod findings;
pub mod formatter;
pub mod git;
pub mod kubernetes;
pub mod linux;
pub mod logger;
pub mod metrics;
pub mod networking;
pub mod policy;
pub mod policy_registry;
pub mod reporting;
pub mod scanner;
pub mod secrets;
pub mod severity;
pub mod terraform;
pub mod yaml;

#[cfg(test)]
mod tests;

// ── Public re-exports ──────────────────────────────────────────────────────────
pub use engine::SecurityEngine;
pub use errors::{SecurityError, SecurityResult};
pub use findings::SecurityFinding;
pub use reporting::SecurityReport;
pub use severity::SecuritySeverity;
pub use category::SecurityCategory;
pub use formatter::ReportFormat;
