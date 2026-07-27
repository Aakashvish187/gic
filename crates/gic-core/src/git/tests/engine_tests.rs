use crate::git::engine::GitEngine;
use git2::Repository;
use tempfile::TempDir;

#[test]
fn test_engine_end_to_end_flow() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path();
    let raw_repo = Repository::init(repo_path).unwrap();

    let file_path = repo_path.join("Dockerfile");
    std::fs::write(&file_path, "FROM ubuntu:22.04\nCMD [\"bash\"]\n").unwrap();

    let mut index = raw_repo.index().unwrap();
    index.add_path(std::path::Path::new("Dockerfile")).unwrap();
    index.write().unwrap();
    let tree_id = index.write_tree().unwrap();
    let tree = raw_repo.find_tree(tree_id).unwrap();
    let sig = raw_repo.signature().unwrap();
    raw_repo
        .commit(Some("HEAD"), &sig, &sig, "Add Dockerfile", &tree, &[])
        .unwrap();

    let engine = GitEngine::new();
    let repo = engine.open_repository(repo_path).unwrap();

    let branch = engine.current_branch(&repo).unwrap();
    assert!(branch.name == "master" || branch.name == "main");

    let new_content = "FROM ubuntu:latest\nCMD [\"bash\"]\nEXPOSE 8080\n";
    let diff = engine
        .compute_buffer_diff(&repo, "Dockerfile", new_content, None)
        .unwrap();

    assert_eq!(diff.modified_count, 1);
    assert_eq!(diff.added_count, 1);

    let decorations = engine.gutter_decorations(&diff);
    assert_eq!(decorations.len(), 2);
}
