//! Thread-safe cache system for Git Awareness Engine.

use crate::git::branch::GitBranch;
use crate::git::diff::FileDiff;
use crate::git::status::RepositoryStatus;
use dashmap::DashMap;
use std::path::PathBuf;
use std::sync::Arc;

/// Cached repository metadata entry.
#[derive(Debug, Clone)]
pub struct CachedRepoData {
    pub branch: GitBranch,
    pub status: RepositoryStatus,
    pub timestamp_ms: u64,
}

/// Thread-safe cache storing Git status and diff calculations.
#[derive(Debug, Clone, Default)]
pub struct GitCache {
    /// Maps repository root path to cached repo data.
    repo_cache: Arc<DashMap<PathBuf, CachedRepoData>>,
    /// Maps file path to cached buffer diff.
    diff_cache: Arc<DashMap<PathBuf, FileDiff>>,
}

impl GitCache {
    pub fn new() -> Self {
        Self {
            repo_cache: Arc::new(DashMap::new()),
            diff_cache: Arc::new(DashMap::new()),
        }
    }

    pub fn get_repo_data(&self, repo_root: &PathBuf) -> Option<CachedRepoData> {
        self.repo_cache.get(repo_root).map(|r| r.value().clone())
    }

    pub fn put_repo_data(&self, repo_root: PathBuf, data: CachedRepoData) {
        self.repo_cache.insert(repo_root, data);
    }

    pub fn get_diff(&self, file_path: &PathBuf) -> Option<FileDiff> {
        self.diff_cache.get(file_path).map(|r| r.value().clone())
    }

    pub fn put_diff(&self, file_path: PathBuf, diff: FileDiff) {
        self.diff_cache.insert(file_path, diff);
    }

    pub fn invalidate_repo(&self, repo_root: &PathBuf) {
        self.repo_cache.remove(repo_root);
        self.diff_cache
            .retain(|path, _| !path.starts_with(repo_root));
    }

    pub fn clear(&self) {
        self.repo_cache.clear();
        self.diff_cache.clear();
    }
}
