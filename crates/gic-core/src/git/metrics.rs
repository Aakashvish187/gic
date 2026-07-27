//! Performance metrics and operational counters for Git Awareness Engine.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// Operational metrics for Git engine operations.
#[derive(Debug, Clone, Default)]
pub struct GitMetrics {
    repo_detections: Arc<AtomicU64>,
    status_scans: Arc<AtomicU64>,
    diff_computations: Arc<AtomicU64>,
    cache_hits: Arc<AtomicU64>,
    cache_misses: Arc<AtomicU64>,
}

impl GitMetrics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn inc_repo_detections(&self) {
        self.repo_detections.fetch_add(1, Ordering::Relaxed);
    }

    pub fn inc_status_scans(&self) {
        self.status_scans.fetch_add(1, Ordering::Relaxed);
    }

    pub fn inc_diff_computations(&self) {
        self.diff_computations.fetch_add(1, Ordering::Relaxed);
    }

    pub fn inc_cache_hits(&self) {
        self.cache_hits.fetch_add(1, Ordering::Relaxed);
    }

    pub fn inc_cache_misses(&self) {
        self.cache_misses.fetch_add(1, Ordering::Relaxed);
    }

    pub fn repo_detections(&self) -> u64 {
        self.repo_detections.load(Ordering::Relaxed)
    }

    pub fn status_scans(&self) -> u64 {
        self.status_scans.load(Ordering::Relaxed)
    }

    pub fn diff_computations(&self) -> u64 {
        self.diff_computations.load(Ordering::Relaxed)
    }

    pub fn cache_hits(&self) -> u64 {
        self.cache_hits.load(Ordering::Relaxed)
    }

    pub fn cache_misses(&self) -> u64 {
        self.cache_misses.load(Ordering::Relaxed)
    }

    pub fn hit_ratio(&self) -> f64 {
        let hits = self.cache_hits();
        let total = hits + self.cache_misses();
        if total == 0 {
            0.0
        } else {
            hits as f64 / total as f64
        }
    }
}
