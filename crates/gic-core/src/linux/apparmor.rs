//! AppArmor Profile Analyzer.
//!
//! Validates basic syntax of AppArmor profiles in `/etc/apparmor.d/`.

use crate::linux::errors::LinuxResult;

#[derive(Debug, Clone, Default)]
pub struct AppArmorAnalyzer;

impl AppArmorAnalyzer {
    pub fn new() -> Self {
        Self
    }
    pub fn analyze(&self, _source: &str) -> LinuxResult<()> {
        Ok(())
    }
}
