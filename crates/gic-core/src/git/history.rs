//! Read-only Git commit history component.

use crate::git::errors::GitResult;
use crate::git::repository::GitRepository;
use std::path::Path;

/// Metadata describing a Git commit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitCommit {
    /// Full 40-character hex SHA hash.
    pub oid: String,
    /// Short 7-character commit hash.
    pub short_oid: String,
    /// Author name.
    pub author_name: String,
    /// Author email address.
    pub author_email: String,
    /// Commit timestamp (seconds since Unix epoch).
    pub timestamp: i64,
    /// First line of commit message.
    pub summary: String,
    /// Full commit message.
    pub message: String,
}

/// History query engine.
#[derive(Debug, Clone, Default)]
pub struct HistoryEngine;

impl HistoryEngine {
    pub fn new() -> Self {
        Self
    }

    /// Fetches latest commit on HEAD.
    pub fn head_commit(&self, repo: &GitRepository) -> GitResult<Option<GitCommit>> {
        let raw = repo.raw_repo();
        let head = match raw.head().and_then(|h| h.peel_to_commit()) {
            Ok(c) => c,
            Err(_) => return Ok(None),
        };

        let oid = head.id().to_string();
        let short_oid = if oid.len() >= 7 {
            oid[..7].to_string()
        } else {
            oid.clone()
        };
        let author = head.author();

        Ok(Some(GitCommit {
            oid,
            short_oid,
            author_name: author.name().unwrap_or("Unknown").to_string(),
            author_email: author.email().unwrap_or("").to_string(),
            timestamp: head.time().seconds(),
            summary: head.summary().unwrap_or("").to_string(),
            message: head.message().unwrap_or("").to_string(),
        }))
    }

    /// Fetches recent commit history for a specific file or full repository.
    pub fn recent_commits<P: AsRef<Path>>(
        &self,
        repo: &GitRepository,
        relative_path: Option<P>,
        limit: usize,
    ) -> GitResult<Vec<GitCommit>> {
        let raw = repo.raw_repo();
        let mut revwalk = raw.revwalk()?;
        if revwalk.push_head().is_err() {
            return Ok(vec![]);
        }

        let mut commits = Vec::new();
        for id in revwalk {
            if commits.len() >= limit {
                break;
            }
            let oid = id?;
            let commit = raw.find_commit(oid)?;

            if let Some(ref rel_path) = relative_path {
                let rel = rel_path.as_ref();
                let tree = commit.tree()?;
                if tree.get_path(rel).is_err() {
                    continue;
                }
            }

            let full_oid = commit.id().to_string();
            let short_oid = if full_oid.len() >= 7 {
                full_oid[..7].to_string()
            } else {
                full_oid.clone()
            };
            let author = commit.author();

            commits.push(GitCommit {
                oid: full_oid,
                short_oid,
                author_name: author.name().unwrap_or("Unknown").to_string(),
                author_email: author.email().unwrap_or("").to_string(),
                timestamp: commit.time().seconds(),
                summary: commit.summary().unwrap_or("").to_string(),
                message: commit.message().unwrap_or("").to_string(),
            });
        }

        Ok(commits)
    }
}
