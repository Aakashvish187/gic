//! Line-level git blame data model and query interfaces for hover tooltips.

use crate::git::errors::GitResult;
use crate::git::repository::GitRepository;
use git2::BlameOptions;
use std::path::Path;

/// Metadata describing author and commit for a line region.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlameHunk {
    /// 1-based start line in target file.
    pub start_line: usize,
    /// Number of lines in this hunk.
    pub line_count: usize,
    /// Commit hash.
    pub commit_oid: String,
    /// Short 7-character hash.
    pub short_oid: String,
    /// Author name.
    pub author_name: String,
    /// Author email address.
    pub author_email: String,
    /// Commit timestamp (seconds since epoch).
    pub timestamp: i64,
    /// Summary line of commit.
    pub summary: String,
}

/// Blame engine interface.
#[derive(Debug, Clone, Default)]
pub struct BlameEngine;

impl BlameEngine {
    pub fn new() -> Self {
        Self
    }

    /// Computes blame hunks for a file path relative to repo root.
    pub fn blame_file<P: AsRef<Path>>(
        &self,
        repo: &GitRepository,
        relative_path: P,
    ) -> GitResult<Vec<BlameHunk>> {
        let rel_path = relative_path.as_ref();
        let raw = repo.raw_repo();

        let mut opts = BlameOptions::new();
        let blame = match raw.blame_file(rel_path, Some(&mut opts)) {
            Ok(b) => b,
            Err(_) => return Ok(vec![]),
        };

        let mut hunks = Vec::with_capacity(blame.len());
        for hunk in blame.iter() {
            let sig = hunk.final_signature();
            let commit_id = hunk.final_commit_id();
            let oid = commit_id.to_string();
            let short_oid = if oid.len() >= 7 {
                oid[..7].to_string()
            } else {
                oid.clone()
            };

            let summary = raw
                .find_commit(commit_id)
                .ok()
                .and_then(|c| c.summary().map(|s| s.to_string()))
                .unwrap_or_default();

            hunks.push(BlameHunk {
                start_line: hunk.final_start_line(),
                line_count: hunk.lines_in_hunk(),
                commit_oid: oid,
                short_oid,
                author_name: sig.name().unwrap_or("Unknown").to_string(),
                author_email: sig.email().unwrap_or("").to_string(),
                timestamp: sig.when().seconds(),
                summary,
            });
        }

        Ok(hunks)
    }

    /// Returns blame hunk for a specific 1-based line number.
    pub fn blame_line<P: AsRef<Path>>(
        &self,
        repo: &GitRepository,
        relative_path: P,
        line_number: usize,
    ) -> GitResult<Option<BlameHunk>> {
        let hunks = self.blame_file(repo, relative_path)?;
        for hunk in hunks {
            if line_number >= hunk.start_line && line_number < hunk.start_line + hunk.line_count {
                return Ok(Some(hunk));
            }
        }
        Ok(None)
    }
}
