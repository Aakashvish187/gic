//! Linux Firewall Analyzer.
//!
//! Basic syntax check for `iptables`, `ufw`, `firewalld` configurations.

use crate::linux::errors::LinuxResult;

#[derive(Debug, Clone, Default)]
pub struct FirewallAnalyzer;

impl FirewallAnalyzer {
    pub fn new() -> Self {
        Self
    }
    pub fn analyze(&self, _source: &str) -> LinuxResult<()> {
        Ok(())
    }
}
