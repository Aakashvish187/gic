//! Evidence data structures backing security findings.

use crate::diagnostics::DiagnosticRange;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Physical evidence supporting a security finding in source code or configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FindingEvidence {
    /// Source file path where evidence was identified (if applicable).
    pub file_path: Option<PathBuf>,
    /// Range of lines/columns in source code.
    pub range: DiagnosticRange,
    /// Code or configuration snippet containing the issue (secrets are sanitized).
    pub snippet: String,
    /// Identifier of the underlying rule or scanner that generated this evidence.
    pub rule_id: String,
    /// Name of the originating infrastructure engine (e.g., "DockerEngine", "SecretScanner").
    pub source_engine: String,
}
