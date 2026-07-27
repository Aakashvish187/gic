//! Policy Registry managing security rules and enforcement thresholds.

use crate::security::category::SecurityCategory;
use crate::security::policy::SecurityPolicy;
use crate::security::severity::SecuritySeverity;
use std::collections::HashMap;

/// Registry storing and querying security policies.
#[derive(Debug, Clone)]
pub struct PolicyRegistry {
    policies: HashMap<String, SecurityPolicy>,
}

impl Default for PolicyRegistry {
    fn default() -> Self {
        let mut registry = Self::new();
        registry.register_defaults();
        registry
    }
}

impl PolicyRegistry {
    pub fn new() -> Self {
        Self {
            policies: HashMap::new(),
        }
    }

    pub fn register(&mut self, policy: SecurityPolicy) {
        self.policies.insert(policy.id.clone(), policy);
    }

    pub fn get(&self, id: &str) -> Option<&SecurityPolicy> {
        self.policies.get(id)
    }

    pub fn list(&self) -> Vec<&SecurityPolicy> {
        self.policies.values().collect()
    }

    fn register_defaults(&mut self) {
        self.register(SecurityPolicy::new(
            "POL-SECRETS",
            "Zero Hardcoded Secrets",
            SecurityCategory::Secrets,
            SecuritySeverity::High,
        ));
        self.register(SecurityPolicy::new(
            "POL-CONTAINERS",
            "Non-Root Container Execution",
            SecurityCategory::Containers,
            SecuritySeverity::Medium,
        ));
        self.register(SecurityPolicy::new(
            "POL-K8S",
            "Kubernetes Pod Security Context",
            SecurityCategory::Kubernetes,
            SecuritySeverity::High,
        ));
        self.register(SecurityPolicy::new(
            "POL-TERRAFORM",
            "Terraform Secure Infrastructure",
            SecurityCategory::Terraform,
            SecuritySeverity::Medium,
        ));
        self.register(SecurityPolicy::new(
            "POL-LINUX",
            "Linux System Hardening",
            SecurityCategory::Linux,
            SecuritySeverity::Medium,
        ));
    }
}
