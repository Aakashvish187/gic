//! Security findings reporting engine — risk scoring, summaries, and affected resource lists.

use crate::security::category::SecurityCategory;
use crate::security::findings::SecurityFinding;
use crate::security::severity::SecuritySeverity;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Overall risk score on a 0–100 scale.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct RiskScore(pub u32);

impl RiskScore {
    pub fn value(&self) -> u32 {
        self.0
    }

    /// Risk rating label for the score.
    pub fn label(&self) -> &'static str {
        match self.0 {
            0..=19 => "Minimal",
            20..=39 => "Low",
            40..=59 => "Medium",
            60..=79 => "High",
            80..=100 => "Critical",
            _ => "Critical",
        }
    }
}

impl std::fmt::Display for RiskScore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/100 ({})", self.0, self.label())
    }
}

/// Count of findings per severity level.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SeverityCounts {
    pub critical: usize,
    pub high: usize,
    pub medium: usize,
    pub low: usize,
    pub information: usize,
}

impl SeverityCounts {
    pub fn total(&self) -> usize {
        self.critical + self.high + self.medium + self.low + self.information
    }
}

/// Complete security report for a repository or file session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityReport {
    /// Computed repository risk score (0–100).
    pub risk_score: RiskScore,
    /// Total findings grouped by severity.
    pub severity_counts: SeverityCounts,
    /// Findings grouped by security category.
    pub category_counts: HashMap<String, usize>,
    /// All security findings.
    pub findings: Vec<SecurityFinding>,
    /// Unique affected file paths.
    pub affected_files: Vec<PathBuf>,
    /// Timestamp when report was generated.
    pub generated_at_ms: u64,
}

/// Report generator building `SecurityReport` from collected findings.
#[derive(Debug, Clone, Default)]
pub struct SecurityReporter;

impl SecurityReporter {
    pub fn new() -> Self {
        Self
    }

    /// Builds a `SecurityReport` from all gathered `SecurityFinding` instances.
    pub fn build_report(&self, findings: Vec<SecurityFinding>) -> SecurityReport {
        let mut severity_counts = SeverityCounts::default();
        let mut category_counts: HashMap<String, usize> = HashMap::new();
        let mut affected_files: Vec<PathBuf> = Vec::new();
        let mut total_weight: u64 = 0;
        let mut max_weight: u64 = 0;

        for finding in &findings {
            let w = finding.severity.risk_weight() as u64;
            total_weight += w;
            max_weight += 10; // max possible per finding is 10 (Critical)

            match finding.severity {
                SecuritySeverity::Critical => severity_counts.critical += 1,
                SecuritySeverity::High => severity_counts.high += 1,
                SecuritySeverity::Medium => severity_counts.medium += 1,
                SecuritySeverity::Low => severity_counts.low += 1,
                SecuritySeverity::Information => severity_counts.information += 1,
            }

            let cat_key = finding.category.display_name().to_string();
            *category_counts.entry(cat_key).or_insert(0) += 1;

            if let Some(ref path) = finding.evidence.file_path {
                if !affected_files.contains(path) {
                    affected_files.push(path.clone());
                }
            }
        }

        // Normalize risk score to 0–100
        let risk_score = if max_weight == 0 {
            RiskScore(0)
        } else {
            RiskScore(((total_weight * 100) / max_weight.max(1)).min(100) as u32)
        };

        let generated_at_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        SecurityReport {
            risk_score,
            severity_counts,
            category_counts,
            findings,
            affected_files,
            generated_at_ms,
        }
    }
}
