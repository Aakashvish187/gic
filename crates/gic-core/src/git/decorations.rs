//! Editor decorations and Status Bar metadata provider APIs.

use crate::git::branch::GitBranch;
use crate::git::diff::{FileDiff, LineChangeKind};
use crate::git::status::{FileStatus, RepositoryStatus};
use std::collections::HashMap;

/// Visual decoration kind for editor line gutter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GutterDecoration {
    /// Green vertical bar for newly added line.
    AddedLine,
    /// Red triangle / marker for deleted line.
    DeletedLine,
    /// Blue vertical bar for modified line.
    ModifiedLine,
}

/// Line decoration entry containing 1-based line number and gutter marker.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LineDecoration {
    pub line_number: usize,
    pub decoration: GutterDecoration,
}

/// Consolidated Status Bar payload for rendering active Git details in the UI.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusBarGitInfo {
    /// Repository name (root folder name).
    pub repo_name: String,
    /// Current branch display string (e.g. "main", "HEAD (a1b2c3d)").
    pub branch_name: String,
    /// Added lines count for current buffer.
    pub added_lines: usize,
    /// Deleted lines count for current buffer.
    pub deleted_lines: usize,
    /// Modified lines count for current buffer.
    pub modified_lines: usize,
    /// Active file status (Modified, Staged, Untracked, Clean, etc.).
    pub file_status: FileStatus,
    /// Total count of modified files across repository.
    pub repo_modified_files_count: usize,
    /// True if repository has uncommitted or untracked changes.
    pub is_repo_dirty: bool,
}

/// Provider for converting diffs and status into UI decorations.
#[derive(Debug, Clone, Default)]
pub struct EditorDecorations;

impl EditorDecorations {
    pub fn new() -> Self {
        Self
    }

    /// Converts a `FileDiff` into a map of line numbers (1-based) to `GutterDecoration`.
    pub fn line_gutter_decorations(&self, diff: &FileDiff) -> HashMap<usize, GutterDecoration> {
        let mut map = HashMap::new();
        for (line_no, change_kind) in &diff.line_changes {
            let gutter = match change_kind {
                LineChangeKind::Added => GutterDecoration::AddedLine,
                LineChangeKind::Deleted => GutterDecoration::DeletedLine,
                LineChangeKind::Modified => GutterDecoration::ModifiedLine,
                LineChangeKind::Unchanged => continue,
            };
            map.insert(*line_no, gutter);
        }
        map
    }

    /// Assembles full `StatusBarGitInfo` from current repository, branch, status, and file diff.
    pub fn build_status_bar_info(
        &self,
        repo_name: String,
        branch: &GitBranch,
        repo_status: &RepositoryStatus,
        file_status: FileStatus,
        file_diff: Option<&FileDiff>,
    ) -> StatusBarGitInfo {
        let (added, deleted, modified) = match file_diff {
            Some(diff) => (diff.added_count, diff.deleted_count, diff.modified_count),
            None => (0, 0, 0),
        };

        StatusBarGitInfo {
            repo_name,
            branch_name: branch.name.clone(),
            added_lines: added,
            deleted_lines: deleted,
            modified_lines: modified,
            file_status,
            repo_modified_files_count: repo_status.modified_count,
            is_repo_dirty: repo_status.is_dirty,
        }
    }
}
