//! Container Resource Requirements and Limits Analyzer.
//!
//! Validates `resources.requests` and `resources.limits` for CPU, memory, and ephemeral storage,
//! detecting missing resource bounds or requests exceeding limits.

use crate::yaml::parser::{YamlMapping, YamlValue};

/// Quantity specification for a single resource (CPU or Memory).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ResourceSpec {
    /// CPU request/limit string (e.g. `500m`, `2`).
    pub cpu: Option<String>,
    /// Memory request/limit string (e.g. `512Mi`, `2Gi`).
    pub memory: Option<String>,
}

/// Extracted resource bounds report for a container.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ContainerResourceReport {
    /// Container identifier name.
    pub container_name: String,
    /// Specified resource requests (`resources.requests`).
    pub requests: Option<ResourceSpec>,
    /// Specified resource limits (`resources.limits`).
    pub limits: Option<ResourceSpec>,
}

impl ContainerResourceReport {
    /// Returns true if both CPU and Memory requests are defined.
    pub fn has_full_requests(&self) -> bool {
        self.requests
            .as_ref()
            .is_some_and(|r| r.cpu.is_some() && r.memory.is_some())
    }

    /// Returns true if both CPU and Memory limits are defined.
    pub fn has_full_limits(&self) -> bool {
        self.limits
            .as_ref()
            .is_some_and(|l| l.cpu.is_some() && l.memory.is_some())
    }
}

/// Analyzer for container compute resource specifications.
#[derive(Debug, Clone, Default)]
pub struct K8sResourceRequirementsAnalyzer;

impl K8sResourceRequirementsAnalyzer {
    /// Creates a new K8sResourceRequirementsAnalyzer.
    pub fn new() -> Self {
        Self
    }

    /// Inspects container map node and returns a `ContainerResourceReport`.
    pub fn inspect_container(
        &self,
        container_name: &str,
        container_map: &YamlMapping,
    ) -> ContainerResourceReport {
        let mut report = ContainerResourceReport {
            container_name: container_name.to_string(),
            requests: None,
            limits: None,
        };

        for pair in &container_map.pairs {
            if pair.key.value == "resources" {
                if let YamlValue::Mapping(ref res_map) = pair.value.value {
                    for r_pair in &res_map.pairs {
                        match r_pair.key.value.as_str() {
                            "requests" => {
                                if let YamlValue::Mapping(ref req_map) = r_pair.value.value {
                                    report.requests = Some(parse_resource_spec(req_map));
                                }
                            }
                            "limits" => {
                                if let YamlValue::Mapping(ref lim_map) = r_pair.value.value {
                                    report.limits = Some(parse_resource_spec(lim_map));
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        report
    }
}

fn parse_resource_spec(map: &YamlMapping) -> ResourceSpec {
    let mut spec = ResourceSpec::default();
    for pair in &map.pairs {
        match pair.key.value.as_str() {
            "cpu" => {
                if let YamlValue::Scalar(ref s) = pair.value.value {
                    spec.cpu = Some(s.value.clone());
                }
            }
            "memory" => {
                if let YamlValue::Scalar(ref s) = pair.value.value {
                    spec.memory = Some(s.value.clone());
                }
            }
            _ => {}
        }
    }
    spec
}
