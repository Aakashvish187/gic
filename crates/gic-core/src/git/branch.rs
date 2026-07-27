//! Git branch inspection and HEAD state details.

use crate::git::errors::{GitError, GitResult};
use crate::git::repository::GitRepository;
use git2::BranchType;

/// Type of Git branch reference.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BranchKind {
    Local,
    Remote,
    Detached,
}

/// Metadata describing current Git branch state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitBranch {
    /// Branch name (e.g., "main", "feature/auth") or short commit hash if HEAD is detached.
    pub name: String,
    /// Kind of branch reference.
    pub kind: BranchKind,
    /// Commit ID hash of HEAD (40-char hex string).
    pub commit_oid: String,
    /// Short 7-char commit hash.
    pub short_oid: String,
    /// Name of upstream remote tracking branch (if configured).
    pub upstream: Option<String>,
    /// Number of commits ahead of upstream.
    pub ahead: usize,
    /// Number of commits behind upstream.
    pub behind: usize,
}

/// Branch resolution helper.
#[derive(Debug, Clone, Default)]
pub struct BranchEngine;

impl BranchEngine {
    pub fn new() -> Self {
        Self
    }

    /// Fetches current active branch information for a repository.
    pub fn current_branch(&self, repo: &GitRepository) -> GitResult<GitBranch> {
        let raw = repo.raw_repo();

        let head = match raw.head() {
            Ok(head) => head,
            Err(e) if e.code() == git2::ErrorCode::UnbornBranch => {
                // Empty repository with no commits yet
                return Ok(GitBranch {
                    name: "main (unborn)".to_string(),
                    kind: BranchKind::Local,
                    commit_oid: "0000000000000000000000000000000000000000".to_string(),
                    short_oid: "0000000".to_string(),
                    upstream: None,
                    ahead: 0,
                    behind: 0,
                });
            }
            Err(e) => return Err(GitError::Git2Error(e)),
        };

        let commit_oid = head.target().map(|o| o.to_string()).unwrap_or_default();
        let short_oid = if commit_oid.len() >= 7 {
            commit_oid[..7].to_string()
        } else {
            commit_oid.clone()
        };

        if raw.head_detached().unwrap_or(false) {
            return Ok(GitBranch {
                name: format!("HEAD ({short_oid})"),
                kind: BranchKind::Detached,
                commit_oid,
                short_oid,
                upstream: None,
                ahead: 0,
                behind: 0,
            });
        }

        let name = head.shorthand().unwrap_or("HEAD").to_string();

        let mut upstream_name = None;
        let mut ahead = 0;
        let mut behind = 0;

        if let Ok(branch) = raw.find_branch(&name, BranchType::Local) {
            if let Ok(upstream) = branch.upstream() {
                if let Some(up_name) = upstream.name().ok().flatten() {
                    upstream_name = Some(up_name.to_string());
                }

                if let (Some(local_oid), Some(upstream_oid)) =
                    (head.target(), upstream.get().target())
                {
                    if let Ok((a, b)) = raw.graph_ahead_behind(local_oid, upstream_oid) {
                        ahead = a;
                        behind = b;
                    }
                }
            }
        }

        Ok(GitBranch {
            name,
            kind: BranchKind::Local,
            commit_oid,
            short_oid,
            upstream: upstream_name,
            ahead,
            behind,
        })
    }
}
