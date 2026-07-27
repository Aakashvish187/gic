//! Central Security Finding representation.

use crate::security::category::SecurityCategory;
use crate::security::evidence::FindingEvidence;
use crate::security::severity::SecuritySeverity;
use serde::{Deserialize, Serialize};

/// Unique identifier for a security finding.
pub type FindingId = String;

/// Central normalized Security Finding produced by DevSecOps Engine.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecurityFinding {
    /// Unique hash identifier for deduplication and tracking.
    pub id: FindingId,
    /// Severity level.
    pub severity: SecuritySeverity,
    /// Security domain category.
    pub category: SecurityCategory,
    /// Concise title summary of finding.
    pub title: String,
    /// Detailed description and security risk explanation.
    pub description: String,
    /// Concrete evidence supporting finding location and context.
    pub evidence: FindingEvidence,
    /// Suggested remediation guidance to fix issue.
    pub remediation: String,
    /// Timestamp when finding was generated (ms since Unix epoch).
    pub timestamp: u64,
}

impl SecurityFinding {
    /// Creates a new `SecurityFinding` instance.
    pub fn new(
        severity: SecuritySeverity,
        category: SecurityCategory,
        title: impl Into<String>,
        description: impl Into<String>,
        evidence: FindingEvidence,
        remediation: impl Into<String>,
    ) -> Self {
        let title_str = title.into();
        let desc_str = description.into();
        let path_str = evidence
            .file_path
            .as_ref()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();

        let raw_id = format!(
            "{}:{}:{}:{}:{}",
            evidence.rule_id,
            path_str,
            evidence.range.start.line,
            evidence.range.start.column,
            title_str
        );

        let id = format!("{:x}", md5_hash(raw_id.as_bytes()));
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        Self {
            id,
            severity,
            category,
            title: title_str,
            description: desc_str,
            evidence,
            remediation: remediation.into(),
            timestamp,
        }
    }
}

/// Simple fallback hashing for unique finding ID generation.
fn md5_hash(bytes: &[u8]) -> u128 {
    let mut hash: u128 = 0xcbf29ce484222325;
    for byte in bytes {
        hash ^= *byte as u128;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}
