//! Operational metrics for Security Intelligence Engine.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// Atomic counters for security engine operations.
#[derive(Debug, Clone, Default)]
pub struct SecurityMetrics {
    scans_performed: Arc<AtomicU64>,
    secrets_detected: Arc<AtomicU64>,
    findings_generated: Arc<AtomicU64>,
    reports_built: Arc<AtomicU64>,
    cache_hits: Arc<AtomicU64>,
    cache_misses: Arc<AtomicU64>,
}

impl SecurityMetrics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn inc_scans(&self) {
        self.scans_performed.fetch_add(1, Ordering::Relaxed);
    }

    pub fn inc_secrets(&self) {
        self.secrets_detected.fetch_add(1, Ordering::Relaxed);
    }

    pub fn inc_findings(&self, count: u64) {
        self.findings_generated.fetch_add(count, Ordering::Relaxed);
    }

    pub fn inc_reports(&self) {
        self.reports_built.fetch_add(1, Ordering::Relaxed);
    }

    pub fn inc_cache_hits(&self) {
        self.cache_hits.fetch_add(1, Ordering::Relaxed);
    }

    pub fn inc_cache_misses(&self) {
        self.cache_misses.fetch_add(1, Ordering::Relaxed);
    }

    pub fn scans(&self) -> u64 { self.scans_performed.load(Ordering::Relaxed) }
    pub fn secrets(&self) -> u64 { self.secrets_detected.load(Ordering::Relaxed) }
    pub fn findings(&self) -> u64 { self.findings_generated.load(Ordering::Relaxed) }
    pub fn reports(&self) -> u64 { self.reports_built.load(Ordering::Relaxed) }
    pub fn cache_hits(&self) -> u64 { self.cache_hits.load(Ordering::Relaxed) }
    pub fn cache_misses(&self) -> u64 { self.cache_misses.load(Ordering::Relaxed) }

    pub fn hit_ratio(&self) -> f64 {
        let h = self.cache_hits();
        let t = h + self.cache_misses();
        if t == 0 { 0.0 } else { h as f64 / t as f64 }
    }
}
