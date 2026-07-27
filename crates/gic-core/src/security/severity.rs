//! Severity levels for security findings and risk scoring calculations.

use serde::{Deserialize, Serialize};

/// Security severity level classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum SecuritySeverity {
    /// Informational notice or best-practice suggestion (weight: 1).
    Information,
    /// Low-risk issue with minor impact (weight: 2).
    Low,
    /// Medium-risk issue requiring planned remediation (weight: 4).
    Medium,
    /// High-risk issue representing an active vulnerability (weight: 7).
    High,
    /// Critical security flaw or exposed secret needing immediate fix (weight: 10).
    Critical,
}

impl SecuritySeverity {
    /// Returns numerical risk weight for calculating overall risk scores.
    pub fn risk_weight(&self) -> u32 {
        match self {
            SecuritySeverity::Information => 1,
            SecuritySeverity::Low => 2,
            SecuritySeverity::Medium => 4,
            SecuritySeverity::High => 7,
            SecuritySeverity::Critical => 10,
        }
    }

    /// Returns standard symbol icon for UI display.
    pub fn symbol(&self) -> &'static str {
        match self {
            SecuritySeverity::Information => "ℹ",
            SecuritySeverity::Low => "⚡",
            SecuritySeverity::Medium => "⚠",
            SecuritySeverity::High => "🚨",
            SecuritySeverity::Critical => "🔥",
        }
    }

    /// Returns canonical string name.
    pub fn name(&self) -> &'static str {
        match self {
            SecuritySeverity::Information => "Information",
            SecuritySeverity::Low => "Low",
            SecuritySeverity::Medium => "Medium",
            SecuritySeverity::High => "High",
            SecuritySeverity::Critical => "Critical",
        }
    }
}

impl std::fmt::Display for SecuritySeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}
