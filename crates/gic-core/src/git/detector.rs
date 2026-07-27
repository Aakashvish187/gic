//! Repository discovery and detection logic.

use crate::git::errors::{GitError, GitResult};
use git2::Repository;
use std::path::{Path, PathBuf};

/// Detection results describing a discovered Git repository.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscoveredRepo {
    /// Absolute path to the repository root directory.
    pub root_path: PathBuf,
    /// Absolute path to the `.git` directory (or git file for worktrees/submodules).
    pub git_dir: PathBuf,
    /// True if the repository is bare (no working directory).
    pub is_bare: bool,
    /// True if this directory is inside a worktree.
    pub is_worktree: bool,
    /// True if this directory is inside a git submodule.
    pub is_submodule: bool,
}

/// Detector component for identifying Git repositories in filesystem trees.
#[derive(Debug, Clone, Default)]
pub struct GitDetector;

impl GitDetector {
    pub fn new() -> Self {
        Self
    }

    /// Discovers a Git repository by traversing parent directories from `start_path`.
    pub fn detect<P: AsRef<Path>>(&self, start_path: P) -> GitResult<DiscoveredRepo> {
        let start_path = start_path.as_ref();
        let canonical_path = if start_path.is_file() {
            start_path.parent().unwrap_or(start_path)
        } else {
            start_path
        };

        let repo = Repository::discover(canonical_path)
            .map_err(|_| GitError::RepositoryNotFound(canonical_path.to_path_buf()))?;

        let is_bare = repo.is_bare();
        let git_dir = repo.path().to_path_buf();

        let root_path = if is_bare {
            git_dir.clone()
        } else {
            repo.workdir()
                .ok_or_else(|| {
                    GitError::InvalidRepository(
                        git_dir.clone(),
                        "Missing workdir for non-bare repo".to_string(),
                    )
                })?
                .to_path_buf()
        };

        let is_submodule = repo.submodules().map(|s| !s.is_empty()).unwrap_or(false);
        let is_worktree = git_dir.components().any(|c| c.as_os_str() == "worktrees");

        Ok(DiscoveredRepo {
            root_path,
            git_dir,
            is_bare,
            is_worktree,
            is_submodule,
        })
    }

    /// Quick check to see if a path is inside a Git repository without opening full metadata.
    pub fn is_inside_repo<P: AsRef<Path>>(&self, path: P) -> bool {
        self.detect(path).is_ok()
    }
}
