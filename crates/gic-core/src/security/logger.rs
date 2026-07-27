//! Structured logger for the Security Intelligence Engine.

use tracing::{debug, error, info, trace, warn};

/// Structured logger for security engine events.
#[derive(Debug, Clone, Default)]
pub struct SecurityLogger;

impl SecurityLogger {
    pub fn new() -> Self {
        Self
    }

    pub fn log_scan_started(&self, target: &str) {
        info!(target: "gic::security", target = target, "Security scan started");
    }

    pub fn log_scan_complete(&self, target: &str, findings_count: usize) {
        info!(target: "gic::security", target = target, findings = findings_count, "Security scan complete");
    }

    pub fn log_secret_found(&self, rule_id: &str, file: &str, line: usize) {
        warn!(target: "gic::security", rule = rule_id, file = file, line = line, "Secret detected");
    }

    pub fn log_report_built(&self, risk_score: u32, total: usize) {
        info!(target: "gic::security", risk_score = risk_score, findings = total, "Security report built");
    }

    pub fn log_error(&self, msg: &str) {
        error!(target: "gic::security", "{msg}");
    }
}
