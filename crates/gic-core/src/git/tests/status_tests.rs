use crate::git::repository::GitRepository;
use crate::git::status::StatusEngine;
use git2::Repository;
use tempfile::TempDir;

#[test]
fn test_status_engine_clean_and_modified() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path();
    let raw_repo = Repository::init(repo_path).unwrap();

    let file_path = repo_path.join("deployment.yaml");
    std::fs::write(&file_path, "replicas: 2\n").unwrap();

    // Initial commit
    let mut index = raw_repo.index().unwrap();
    index
        .add_path(std::path::Path::new("deployment.yaml"))
        .unwrap();
    index.write().unwrap();
    let tree_id = index.write_tree().unwrap();
    let tree = raw_repo.find_tree(tree_id).unwrap();
    let sig = raw_repo.signature().unwrap();
    raw_repo
        .commit(Some("HEAD"), &sig, &sig, "Initial commit", &tree, &[])
        .unwrap();

    let git_repo = GitRepository::open_from_path(repo_path).unwrap();
    let engine = StatusEngine::new();

    // Check clean status
    let status_before = engine.repository_status(&git_repo).unwrap();
    assert!(!status_before.is_dirty);

    // Modify file
    std::fs::write(&file_path, "replicas: 5\n").unwrap();

    let status_after = engine.repository_status(&git_repo).unwrap();
    assert!(status_after.is_dirty);
    assert_eq!(status_after.modified_count, 1);
}
