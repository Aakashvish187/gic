//! Security analyzer ingesting domain diagnostics from all engines.

use crate::diagnostics::Diagnostic;
use crate::security::docker::DockerSecurityAdapter;
use crate::security::findings::SecurityFinding;
use crate::security::git::GitSecurityAdapter;
use crate::security::kubernetes::K8sSecurityAdapter;
use crate::security::linux::LinuxSecurityAdapter;
use crate::security::networking::NetworkSecurityAdapter;
use crate::security::terraform::TerraformSecurityAdapter;
use crate::security::yaml::YamlSecurityAdapter;

/// Diagnostic source from a specific infrastructure engine.
#[derive(Debug)]
pub enum EngineDiagnosticSource<'a> {
    Docker(&'a [Diagnostic]),
    Kubernetes(&'a [Diagnostic]),
    Terraform(&'a [Diagnostic]),
    Linux(&'a [Diagnostic]),
    Networking(&'a [Diagnostic]),
    Git(&'a [Diagnostic]),
    Yaml(&'a [Diagnostic]),
}

/// Domain adapter security analyzer collecting findings from all registered engines.
#[derive(Debug, Clone, Default)]
pub struct SecurityAnalyzer {
    docker: DockerSecurityAdapter,
    kubernetes: K8sSecurityAdapter,
    terraform: TerraformSecurityAdapter,
    linux: LinuxSecurityAdapter,
    networking: NetworkSecurityAdapter,
    git: GitSecurityAdapter,
    yaml: YamlSecurityAdapter,
}

impl SecurityAnalyzer {
    pub fn new() -> Self {
        Self::default()
    }

    /// Analyzes diagnostics from a specific engine and returns security findings.
    pub fn analyze_source<'a>(&self, source: EngineDiagnosticSource<'a>) -> Vec<SecurityFinding> {
        match source {
            EngineDiagnosticSource::Docker(diags) => self.docker.convert_diagnostics(diags),
            EngineDiagnosticSource::Kubernetes(diags) => self.kubernetes.convert_diagnostics(diags),
            EngineDiagnosticSource::Terraform(diags) => self.terraform.convert_diagnostics(diags),
            EngineDiagnosticSource::Linux(diags) => self.linux.convert_diagnostics(diags),
            EngineDiagnosticSource::Networking(diags) => self.networking.convert_diagnostics(diags),
            EngineDiagnosticSource::Git(diags) => self.git.convert_diagnostics(diags),
            EngineDiagnosticSource::Yaml(diags) => self.yaml.convert_diagnostics(diags),
        }
    }

    /// Bulk-processes multiple engine sources, returning the unified findings list.
    pub fn analyze_all(&self, sources: Vec<EngineDiagnosticSource<'_>>) -> Vec<SecurityFinding> {
        sources.into_iter().flat_map(|s| self.analyze_source(s)).collect()
    }
}
