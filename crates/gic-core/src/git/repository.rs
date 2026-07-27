//! Representation and metadata querying for a Git repository.

use crate::git::detector::DiscoveredRepo;
use crate::git::errors::GitResult;
use git2::Repository;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Handle for inspecting a Git repository.
#[derive(Clone)]
pub struct GitRepository {
    pub root_path: PathBuf,
    pub git_dir: PathBuf,
    pub is_bare: bool,
    pub is_worktree: bool,
    pub is_submodule: bool,
    pub(crate) inner: Arc<Repository>,
}

impl std::fmt::Debug for GitRepository {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GitRepository")
            .field("root_path", &self.root_path)
            .field("git_dir", &self.git_dir)
            .field("is_bare", &self.is_bare)
            .field("is_worktree", &self.is_worktree)
            .field("is_submodule", &self.is_submodule)
            .finish()
    }
}

impl GitRepository {
    /// Opens a repository from a discovered location.
    pub fn open(info: DiscoveredRepo) -> GitResult<Self> {
        let repo = Repository::open(&info.git_dir)?;

        Ok(Self {
            root_path: info.root_path,
            git_dir: info.git_dir,
            is_bare: info.is_bare,
            is_worktree: info.is_worktree,
            is_submodule: info.is_submodule,
            inner: Arc::new(repo),
        })
    }

    /// Opens a repository from any path inside the repository.
    pub fn open_from_path<P: AsRef<Path>>(path: P) -> GitResult<Self> {
        let detector = crate::git::detector::GitDetector::new();
        let info = detector.detect(path)?;
        Self::open(info)
    }

    /// Returns path relative to repository root for a target absolute path.
    pub fn relative_path<P: AsRef<Path>>(&self, abs_path: P) -> Option<PathBuf> {
        let abs_path = abs_path.as_ref();
        abs_path
            .strip_prefix(&self.root_path)
            .ok()
            .map(|p| p.to_path_buf())
    }

    /// Direct access to underlying libgit2 handle (internal to crate).
    pub(crate) fn raw_repo(&self) -> &Repository {
        &self.inner
    }
}
