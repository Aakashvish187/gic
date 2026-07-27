use crate::git::history::HistoryEngine;
use crate::git::repository::GitRepository;
use git2::Repository;
use tempfile::TempDir;

#[test]
fn test_history_head_commit() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path();
    let raw_repo = Repository::init(repo_path).unwrap();

    let file_path = repo_path.join("README.md");
    std::fs::write(&file_path, "# GIC Project").unwrap();

    let mut index = raw_repo.index().unwrap();
    index.add_path(std::path::Path::new("README.md")).unwrap();
    index.write().unwrap();
    let tree_id = index.write_tree().unwrap();
    let tree = raw_repo.find_tree(tree_id).unwrap();
    let sig = raw_repo.signature().unwrap();
    raw_repo
        .commit(
            Some("HEAD"),
            &sig,
            &sig,
            "Initial commit message",
            &tree,
            &[],
        )
        .unwrap();

    let git_repo = GitRepository::open_from_path(repo_path).unwrap();
    let history = HistoryEngine::new();

    let head = history.head_commit(&git_repo).unwrap().unwrap();
    assert_eq!(head.summary, "Initial commit message");
}
