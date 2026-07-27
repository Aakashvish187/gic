//! Line-by-line diff engine comparing active text buffer against HEAD commit blob.

use crate::git::errors::{GitError, GitResult};
use crate::git::repository::GitRepository;
use git2::{Blob, ObjectType};
use std::path::Path;

/// Kind of change for a specific line in a diff.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LineChangeKind {
    Unchanged,
    Added,
    Deleted,
    Modified,
}

/// Description of a single line inside a diff hunk.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffLine {
    pub old_line_number: Option<usize>,
    pub new_line_number: Option<usize>,
    pub kind: LineChangeKind,
    pub content: String,
}

/// A contiguous chunk of diff changes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffHunk {
    pub old_start: usize,
    pub old_lines: usize,
    pub new_start: usize,
    pub new_lines: usize,
    pub header: String,
    pub lines: Vec<DiffLine>,
}

/// Complete file diff summary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileDiff {
    /// Path relative to repository root.
    pub relative_path: String,
    /// Total added lines count.
    pub added_count: usize,
    /// Total deleted lines count.
    pub deleted_count: usize,
    /// Total modified lines count.
    pub modified_count: usize,
    /// True if file is binary.
    pub is_binary: bool,
    /// Detailed diff hunks.
    pub hunks: Vec<DiffHunk>,
    /// Line-level change mapping indexed by new line number (1-based).
    pub line_changes: Vec<(usize, LineChangeKind)>,
}

/// Diff options for customizing comparison behavior.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffOptions {
    /// Ignore whitespace changes.
    pub ignore_whitespace: bool,
    /// Maximum file size to attempt line diffing (in bytes). Default: 10MB.
    pub max_file_size_bytes: usize,
}

impl Default for DiffOptions {
    fn default() -> Self {
        Self {
            ignore_whitespace: false,
            max_file_size_bytes: 10 * 1024 * 1024,
        }
    }
}

/// Line-by-line Diff Engine.
#[derive(Debug, Clone, Default)]
pub struct DiffEngine;

impl DiffEngine {
    pub fn new() -> Self {
        Self
    }

    /// Fetches HEAD content for a relative path in the repository.
    pub fn fetch_head_content<P: AsRef<Path>>(
        &self,
        repo: &GitRepository,
        relative_path: P,
    ) -> GitResult<Option<String>> {
        let raw = repo.raw_repo();
        let head_commit = match raw.head().and_then(|h| h.peel_to_commit()) {
            Ok(c) => c,
            Err(_) => return Ok(None), // Unborn branch or empty repo
        };

        let tree = head_commit.tree()?;
        let entry = match tree.get_path(relative_path.as_ref()) {
            Ok(e) => e,
            Err(_) => return Ok(None), // New untracked file not in HEAD
        };

        let object = entry.to_object(raw)?;
        if object.kind() != Some(ObjectType::Blob) {
            return Ok(None);
        }

        let blob: Blob = object
            .into_blob()
            .map_err(|_| GitError::DiffError("Failed to convert object to blob".to_string()))?;
        if blob.is_binary() {
            return Ok(None);
        }

        let content = std::str::from_utf8(blob.content())
            .map_err(|_| GitError::DiffError("Invalid UTF-8 in HEAD blob".to_string()))?
            .to_string();

        Ok(Some(content))
    }

    /// Computes diff between active buffer content and HEAD content for a file.
    pub fn compute_buffer_diff<P: AsRef<Path>>(
        &self,
        repo: &GitRepository,
        relative_path: P,
        buffer_content: &str,
        options: &DiffOptions,
    ) -> GitResult<FileDiff> {
        let rel_path_str = relative_path.as_ref().to_string_lossy().to_string();

        if buffer_content.len() > options.max_file_size_bytes {
            return Ok(FileDiff {
                relative_path: rel_path_str,
                added_count: 0,
                deleted_count: 0,
                modified_count: 0,
                is_binary: true,
                hunks: vec![],
                line_changes: vec![],
            });
        }

        let head_content = self
            .fetch_head_content(repo, relative_path)?
            .unwrap_or_default();

        let head_lines: Vec<&str> = head_content.lines().collect();
        let buf_lines: Vec<&str> = buffer_content.lines().collect();

        let mut added_count = 0;
        let mut deleted_count = 0;
        let mut modified_count = 0;
        let mut line_changes = Vec::new();
        let mut diff_lines = Vec::new();

        // High performance line-by-line diff algorithm
        let max_lines = head_lines.len().max(buf_lines.len());
        for i in 0..max_lines {
            let old_line = head_lines.get(i).copied();
            let new_line = buf_lines.get(i).copied();

            match (old_line, new_line) {
                (Some(o), Some(n)) => {
                    let is_match = if options.ignore_whitespace {
                        o.trim() == n.trim()
                    } else {
                        o == n
                    };

                    if is_match {
                        diff_lines.push(DiffLine {
                            old_line_number: Some(i + 1),
                            new_line_number: Some(i + 1),
                            kind: LineChangeKind::Unchanged,
                            content: n.to_string(),
                        });
                    } else {
                        modified_count += 1;
                        line_changes.push((i + 1, LineChangeKind::Modified));
                        diff_lines.push(DiffLine {
                            old_line_number: Some(i + 1),
                            new_line_number: Some(i + 1),
                            kind: LineChangeKind::Modified,
                            content: n.to_string(),
                        });
                    }
                }
                (None, Some(n)) => {
                    added_count += 1;
                    line_changes.push((i + 1, LineChangeKind::Added));
                    diff_lines.push(DiffLine {
                        old_line_number: None,
                        new_line_number: Some(i + 1),
                        kind: LineChangeKind::Added,
                        content: n.to_string(),
                    });
                }
                (Some(o), None) => {
                    deleted_count += 1;
                    line_changes.push((i + 1, LineChangeKind::Deleted));
                    diff_lines.push(DiffLine {
                        old_line_number: Some(i + 1),
                        new_line_number: None,
                        kind: LineChangeKind::Deleted,
                        content: o.to_string(),
                    });
                }
                (None, None) => {}
            }
        }

        let hunk = DiffHunk {
            old_start: 1,
            old_lines: head_lines.len(),
            new_start: 1,
            new_lines: buf_lines.len(),
            header: format!("@@ -1,{} +1,{} @@", head_lines.len(), buf_lines.len()),
            lines: diff_lines,
        };

        Ok(FileDiff {
            relative_path: rel_path_str,
            added_count,
            deleted_count,
            modified_count,
            is_binary: false,
            hunks: vec![hunk],
            line_changes,
        })
    }
}
