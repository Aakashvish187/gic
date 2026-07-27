use crate::git::diff::{DiffEngine, DiffOptions, LineChangeKind};
use crate::git::repository::GitRepository;
use git2::Repository;
use tempfile::TempDir;

#[test]
fn test_diff_engine_line_modifications() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path();
    let raw_repo = Repository::init(repo_path).unwrap();

    let file_name = "deployment.yaml";
    let file_path = repo_path.join(file_name);
    std::fs::write(&file_path, "replicas: 2\nimage: nginx\n").unwrap();

    let mut index = raw_repo.index().unwrap();
    index.add_path(std::path::Path::new(file_name)).unwrap();
    index.write().unwrap();
    let tree_id = index.write_tree().unwrap();
    let tree = raw_repo.find_tree(tree_id).unwrap();
    let sig = raw_repo.signature().unwrap();
    raw_repo
        .commit(Some("HEAD"), &sig, &sig, "Initial commit", &tree, &[])
        .unwrap();

    let git_repo = GitRepository::open_from_path(repo_path).unwrap();
    let engine = DiffEngine::new();

    let new_buffer = "replicas: 5\nimage: nginx\nports: 80\n";
    let diff = engine
        .compute_buffer_diff(&git_repo, file_name, new_buffer, &DiffOptions::default())
        .unwrap();

    assert_eq!(diff.modified_count, 1);
    assert_eq!(diff.added_count, 1);

    let changes = diff.line_changes;
    assert!(changes.contains(&(1, LineChangeKind::Modified)));
    assert!(changes.contains(&(3, LineChangeKind::Added)));
}
