//! Kubernetes Container Probe Analyzer (`livenessProbe`, `readinessProbe`, `startupProbe`).
//!
//! Inspects pod templates for health check configurations and identifies missing or
//! improperly configured container probes.

use crate::yaml::parser::{YamlMapping, YamlValue};

/// Category of container health check probe.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProbeType {
    Liveness,
    Readiness,
    Startup,
}

/// Description of probe configuration.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ProbeConfig {
    /// True if probe action (`httpGet`, `exec`, `tcpSocket`, `grpc`) is defined.
    pub has_handler: bool,
    /// Delay before first probe execution.
    pub initial_delay_seconds: Option<usize>,
    /// Probe interval frequency in seconds.
    pub period_seconds: Option<usize>,
    /// Probe timeout limit in seconds.
    pub timeout_seconds: Option<usize>,
}

/// Summary report of probe configuration for a container.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContainerProbeReport {
    /// Container name.
    pub container_name: String,
    /// Status of livenessProbe.
    pub liveness: Option<ProbeConfig>,
    /// Status of readinessProbe.
    pub readiness: Option<ProbeConfig>,
    /// Status of startupProbe.
    pub startup: Option<ProbeConfig>,
}

/// Analyzer for container health probes.
#[derive(Debug, Clone, Default)]
pub struct K8sProbeAnalyzer;

impl K8sProbeAnalyzer {
    /// Creates a new K8sProbeAnalyzer.
    pub fn new() -> Self {
        Self
    }

    /// Extracts probe configuration report for a container map node.
    pub fn inspect_container(
        &self,
        container_name: &str,
        container_map: &YamlMapping,
    ) -> ContainerProbeReport {
        let mut liveness = None;
        let mut readiness = None;
        let mut startup = None;

        for pair in &container_map.pairs {
            match pair.key.value.as_str() {
                "livenessProbe" => {
                    if let YamlValue::Mapping(ref p_map) = pair.value.value {
                        liveness = Some(parse_probe_map(p_map));
                    }
                }
                "readinessProbe" => {
                    if let YamlValue::Mapping(ref p_map) = pair.value.value {
                        readiness = Some(parse_probe_map(p_map));
                    }
                }
                "startupProbe" => {
                    if let YamlValue::Mapping(ref p_map) = pair.value.value {
                        startup = Some(parse_probe_map(p_map));
                    }
                }
                _ => {}
            }
        }

        ContainerProbeReport {
            container_name: container_name.to_string(),
            liveness,
            readiness,
            startup,
        }
    }
}

fn parse_probe_map(map: &YamlMapping) -> ProbeConfig {
    let mut config = ProbeConfig::default();

    for pair in &map.pairs {
        match pair.key.value.as_str() {
            "httpGet" | "exec" | "tcpSocket" | "grpc" => {
                config.has_handler = true;
            }
            "initialDelaySeconds" => {
                if let YamlValue::Scalar(ref s) = pair.value.value {
                    config.initial_delay_seconds = s.value.parse().ok();
                }
            }
            "periodSeconds" => {
                if let YamlValue::Scalar(ref s) = pair.value.value {
                    config.period_seconds = s.value.parse().ok();
                }
            }
            "timeoutSeconds" => {
                if let YamlValue::Scalar(ref s) = pair.value.value {
                    config.timeout_seconds = s.value.parse().ok();
                }
            }
            _ => {}
        }
    }

    config
}
