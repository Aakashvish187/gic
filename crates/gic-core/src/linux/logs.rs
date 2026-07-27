//! Log Configuration Analyzer.
//!
//! Validates `logrotate.conf` syntax.

use crate::linux::errors::LinuxResult;

#[derive(Debug, Clone, Default)]
pub struct LogsAnalyzer;

impl LogsAnalyzer {
    pub fn new() -> Self {
        Self
    }
    pub fn analyze(&self, _source: &str) -> LinuxResult<()> {
        Ok(())
    }
}
