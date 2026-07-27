//! Structured logging helper for the Git Awareness Engine.

use std::path::Path;
use tracing::{debug, error, info, trace, warn};

/// Thread-safe logger interface for Git subsystem events.
#[derive(Debug, Clone, Default)]
pub struct GitLogger;

impl GitLogger {
    pub fn new() -> Self {
        Self
    }

    pub fn log_repo_detected(&self, root: &Path, is_bare: bool) {
        info!(
            target: "gic::git",
            path = %root.display(),
            is_bare = is_bare,
            "Git repository detected"
        );
    }

    pub fn log_status_scanned(&self, root: &Path, modified_count: usize, total_count: usize) {
        debug!(
            target: "gic::git",
            path = %root.display(),
            modified = modified_count,
            total = total_count,
            "Git status scan completed"
        );
    }

    pub fn log_diff_calculated(
        &self,
        file_path: &Path,
        added: usize,
        deleted: usize,
        modified: usize,
    ) {
        trace!(
            target: "gic::git",
            path = %file_path.display(),
            added = added,
            deleted = deleted,
            modified = modified,
            "Buffer diff calculated"
        );
    }

    pub fn log_warn(&self, msg: &str) {
        warn!(target: "gic::git", "{msg}");
    }

    pub fn log_error(&self, msg: &str) {
        error!(target: "gic::git", "{msg}");
    }
}
