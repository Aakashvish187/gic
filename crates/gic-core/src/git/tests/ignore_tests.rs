use crate::git::ignore::GitIgnoreEngine;
use crate::git::repository::GitRepository;
use git2::Repository;
use tempfile::TempDir;

#[test]
fn test_gitignore_rules() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path();
    Repository::init(repo_path).unwrap();

    let gitignore = repo_path.join(".gitignore");
    std::fs::write(&gitignore, "*.log\ntarget/\n").unwrap();

    let git_repo = GitRepository::open_from_path(repo_path).unwrap();
    let engine = GitIgnoreEngine::new(&git_repo).unwrap();

    assert!(engine.is_ignored(repo_path.join("app.log")));
    assert!(!engine.is_ignored(repo_path.join("main.rs")));
}
