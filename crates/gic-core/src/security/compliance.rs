//! Compliance Framework mapping architecture (CIS, NIST, PCI-DSS, SOC 2, ISO 27001, OWASP, MITRE ATT&CK).

use serde::{Deserialize, Serialize};

/// Supported industry compliance standards and regulatory frameworks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ComplianceFramework {
    CisBenchmarks,
    Nist800_53,
    PciDss,
    Soc2,
    Iso27001,
    OwaspTop10,
    MitreAttck,
}

impl ComplianceFramework {
    pub fn display_name(&self) -> &'static str {
        match self {
            ComplianceFramework::CisBenchmarks => "CIS Benchmarks v8",
            ComplianceFramework::Nist800_53 => "NIST SP 800-53",
            ComplianceFramework::PciDss => "PCI-DSS v4.0",
            ComplianceFramework::Soc2 => "SOC 2 Type II",
            ComplianceFramework::Iso27001 => "ISO/IEC 27001:2022",
            ComplianceFramework::OwaspTop10 => "OWASP Top 10 Security Risks",
            ComplianceFramework::MitreAttck => "MITRE ATT&CK Enterprise",
        }
    }
}

/// Control mapping linking a security rule ID to compliance controls.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComplianceMapping {
    pub framework: ComplianceFramework,
    pub control_id: String,
    pub control_title: String,
}
