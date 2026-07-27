//! Security policy model definitions.

use crate::security::category::SecurityCategory;
use crate::security::severity::SecuritySeverity;
use serde::{Deserialize, Serialize};

/// Custom security policy definition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecurityPolicy {
    /// Policy ID (e.g., "POL-SEC-01").
    pub id: String,
    /// Policy name.
    pub name: String,
    /// Policy description.
    pub description: String,
    /// Security domain category.
    pub category: SecurityCategory,
    /// Enforced minimum severity threshold.
    pub min_severity: SecuritySeverity,
    /// True if policy rule is actively enforced.
    pub enabled: bool,
}

impl SecurityPolicy {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        category: SecurityCategory,
        min_severity: SecuritySeverity,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: String::new(),
            category,
            min_severity,
            enabled: true,
        }
    }
}
