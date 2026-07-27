//! Linux Services Analyzer.
//!
//! Façade for init systems (Systemd, SysV init, OpenRC).

use crate::linux::errors::LinuxResult;
use crate::linux::systemd::{SystemdAnalyzer, SystemdDiagnostic};

/// Services configuration analyzer.
#[derive(Debug, Clone, Default)]
pub struct ServicesAnalyzer {
    systemd: SystemdAnalyzer,
}

impl ServicesAnalyzer {
    pub fn new() -> Self {
        Self {
            systemd: SystemdAnalyzer::new(),
        }
    }

    /// Analyzes a raw systemd unit file string.
    pub fn analyze_systemd(&self, source: &str) -> LinuxResult<Vec<SystemdDiagnostic>> {
        self.systemd.analyze(source)
    }
}
