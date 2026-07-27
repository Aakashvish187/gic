//! Git status classifications for individual files and workspace summary.

use crate::git::errors::GitResult;
use crate::git::repository::GitRepository;
use git2::{Status, StatusOptions};
use std::path::{Path, PathBuf};

/// Status classification of an individual file in a Git repository.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FileStatus {
    /// Unmodified relative to HEAD index and working directory.
    Clean,
    /// File has been modified in working directory (unstaged).
    Modified,
    /// New file added to index (staged).
    Added,
    /// File deleted in working directory.
    Deleted,
    /// File renamed.
    Renamed,
    /// File copied.
    Copied,
    /// File is not tracked by Git.
    Untracked,
    /// File is ignored by .gitignore rules.
    Ignored,
    /// Changes staged for commit.
    Staged,
    /// Both modified or conflicted during merge.
    Conflicted,
}

/// Status summary entry for a single file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileStatusEntry {
    /// Path relative to repository root.
    pub path: PathBuf,
    /// Classified file status.
    pub status: FileStatus,
    /// True if changes are staged in index.
    pub is_staged: bool,
}

/// Overall repository status summary.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RepositoryStatus {
    /// Total count of modified files.
    pub modified_count: usize,
    /// Total count of staged files.
    pub staged_count: usize,
    /// Total count of untracked files.
    pub untracked_count: usize,
    /// Total count of conflicted files.
    pub conflicted_count: usize,
    /// True if repository has any uncommitted or untracked changes.
    pub is_dirty: bool,
    /// Detailed status per file path.
    pub entries: Vec<FileStatusEntry>,
}

/// Engine responsible for computing status for files or entire repository.
#[derive(Debug, Clone, Default)]
pub struct StatusEngine;

impl StatusEngine {
    pub fn new() -> Self {
        Self
    }

    /// Computes full repository status.
    pub fn repository_status(&self, repo: &GitRepository) -> GitResult<RepositoryStatus> {
        let mut opts = StatusOptions::new();
        opts.include_untracked(true)
            .include_ignored(false)
            .recurse_untracked_dirs(true);

        let raw_statuses = repo.raw_repo().statuses(Some(&mut opts))?;

        let mut modified_count = 0;
        let mut staged_count = 0;
        let mut untracked_count = 0;
        let mut conflicted_count = 0;
        let mut entries = Vec::with_capacity(raw_statuses.len());

        for entry in raw_statuses.iter() {
            let path_str = match entry.path() {
                Some(p) => p,
                None => continue,
            };
            let path = PathBuf::from(path_str);
            let s = entry.status();

            let (file_status, is_staged) = classify_git2_status(s);

            match file_status {
                FileStatus::Modified => modified_count += 1,
                FileStatus::Added | FileStatus::Staged => staged_count += 1,
                FileStatus::Untracked => untracked_count += 1,
                FileStatus::Conflicted => conflicted_count += 1,
                _ => {}
            }

            if is_staged {
                staged_count += 1;
            }

            entries.push(FileStatusEntry {
                path,
                status: file_status,
                is_staged,
            });
        }

        let is_dirty =
            modified_count > 0 || staged_count > 0 || untracked_count > 0 || conflicted_count > 0;

        Ok(RepositoryStatus {
            modified_count,
            staged_count,
            untracked_count,
            conflicted_count,
            is_dirty,
            entries,
        })
    }

    /// Computes file status for a single target file path relative to repo root.
    pub fn file_status<P: AsRef<Path>>(
        &self,
        repo: &GitRepository,
        relative_path: P,
    ) -> GitResult<FileStatus> {
        let rel_path = relative_path.as_ref();
        let s = repo.raw_repo().status_file(rel_path)?;
        let (status, _) = classify_git2_status(s);
        Ok(status)
    }
}

/// Helper mapping git2::Status flags to FileStatus enum and staged flag.
fn classify_git2_status(s: Status) -> (FileStatus, bool) {
    if s.is_conflicted() {
        return (FileStatus::Conflicted, false);
    }
    if s.contains(Status::INDEX_NEW)
        || s.contains(Status::INDEX_MODIFIED)
        || s.contains(Status::INDEX_DELETED)
    {
        return (FileStatus::Staged, true);
    }
    if s.contains(Status::WT_NEW) {
        return (FileStatus::Untracked, false);
    }
    if s.contains(Status::WT_MODIFIED) {
        return (FileStatus::Modified, false);
    }
    if s.contains(Status::WT_DELETED) {
        return (FileStatus::Deleted, false);
    }
    if s.contains(Status::WT_RENAMED) || s.contains(Status::INDEX_RENAMED) {
        return (FileStatus::Renamed, false);
    }
    if s.contains(Status::IGNORED) {
        return (FileStatus::Ignored, false);
    }

    (FileStatus::Clean, false)
}
