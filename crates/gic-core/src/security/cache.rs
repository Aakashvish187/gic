//! Thread-safe security findings cache using DashMap.

use crate::security::findings::SecurityFinding;
use crate::security::reporting::SecurityReport;
use dashmap::DashMap;
use std::path::PathBuf;
use std::sync::Arc;

/// Cached file-level scanning entry.
#[derive(Debug, Clone)]
pub struct CachedFileScan {
    pub findings: Vec<SecurityFinding>,
    pub timestamp_ms: u64,
}

/// Thread-safe cache for security findings and reports.
#[derive(Debug, Clone, Default)]
pub struct SecurityCache {
    /// Maps file path to cached scan results.
    file_cache: Arc<DashMap<PathBuf, CachedFileScan>>,
    /// Cached full repository security report.
    report_cache: Arc<DashMap<String, SecurityReport>>,
}

impl SecurityCache {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_file_findings(&self, path: &PathBuf) -> Option<Vec<SecurityFinding>> {
        self.file_cache
            .get(path)
            .map(|e| e.value().findings.clone())
    }

    pub fn put_file_findings(&self, path: PathBuf, findings: Vec<SecurityFinding>) {
        let timestamp_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        self.file_cache.insert(
            path,
            CachedFileScan {
                findings,
                timestamp_ms,
            },
        );
    }

    pub fn get_report(&self, key: &str) -> Option<SecurityReport> {
        self.report_cache.get(key).map(|r| r.value().clone())
    }

    pub fn put_report(&self, key: String, report: SecurityReport) {
        self.report_cache.insert(key, report);
    }

    pub fn invalidate_file(&self, path: &PathBuf) {
        self.file_cache.remove(path);
    }

    pub fn clear(&self) {
        self.file_cache.clear();
        self.report_cache.clear();
    }
}
