use crate::git::branch::{BranchKind, GitBranch};
use crate::git::cache::{CachedRepoData, GitCache};
use crate::git::status::RepositoryStatus;
use std::path::PathBuf;

#[test]
fn test_git_cache_put_get_invalidate() {
    let cache = GitCache::new();
    let repo_root = PathBuf::from("/tmp/myrepo");

    let branch = GitBranch {
        name: "main".to_string(),
        kind: BranchKind::Local,
        commit_oid: "a".repeat(40),
        short_oid: "a".repeat(7),
        upstream: None,
        ahead: 0,
        behind: 0,
    };

    let status = RepositoryStatus::default();

    cache.put_repo_data(
        repo_root.clone(),
        CachedRepoData {
            branch,
            status,
            timestamp_ms: 1000,
        },
    );

    assert!(cache.get_repo_data(&repo_root).is_some());

    cache.invalidate_repo(&repo_root);
    assert!(cache.get_repo_data(&repo_root).is_none());
}
