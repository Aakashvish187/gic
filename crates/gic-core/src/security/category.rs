//! Security categories classifying infrastructure vulnerabilities and risks.

use serde::{Deserialize, Serialize};

/// Categorization of security findings across infrastructure domains.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum SecurityCategory {
    Secrets,
    Credentials,
    Identity,
    AccessControl,
    Networking,
    Containers,
    Kubernetes,
    Terraform,
    Linux,
    Cloud,
    Certificates,
    Encryption,
    Compliance,
    SupplyChain,
    Configuration,
    BestPractices,
}

impl SecurityCategory {
    pub fn display_name(&self) -> &'static str {
        match self {
            SecurityCategory::Secrets => "Secrets & Hardcoded Keys",
            SecurityCategory::Credentials => "Database & Service Credentials",
            SecurityCategory::Identity => "Identity & IAM",
            SecurityCategory::AccessControl => "Access Control & Permissions",
            SecurityCategory::Networking => "Network Security",
            SecurityCategory::Containers => "Container Security",
            SecurityCategory::Kubernetes => "Kubernetes Security",
            SecurityCategory::Terraform => "Terraform Security",
            SecurityCategory::Linux => "Linux Host Security",
            SecurityCategory::Cloud => "Cloud Infrastructure Security",
            SecurityCategory::Certificates => "PKI & Certificate Security",
            SecurityCategory::Encryption => "Data Encryption",
            SecurityCategory::Compliance => "Regulatory Compliance",
            SecurityCategory::SupplyChain => "Supply Chain & Image Pinning",
            SecurityCategory::Configuration => "Security Misconfiguration",
            SecurityCategory::BestPractices => "Infrastructure Best Practices",
        }
    }
}

impl std::fmt::Display for SecurityCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}
