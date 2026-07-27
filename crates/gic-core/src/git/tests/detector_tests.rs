use crate::git::detector::GitDetector;
use git2::Repository;
use tempfile::TempDir;

#[test]
fn test_detect_valid_repository() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path();
    Repository::init(repo_path).unwrap();

    let detector = GitDetector::new();
    let discovered = detector.detect(repo_path).unwrap();

    assert_eq!(
        discovered.root_path.file_name().unwrap(),
        repo_path.file_name().unwrap()
    );
    assert!(!discovered.is_bare);
}

#[test]
fn test_detect_nested_file_in_repository() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path();
    Repository::init(repo_path).unwrap();

    let sub_dir = repo_path.join("src").join("config");
    std::fs::create_dir_all(&sub_dir).unwrap();
    let test_file = sub_dir.join("main.rs");
    std::fs::write(&test_file, "fn main() {}").unwrap();

    let detector = GitDetector::new();
    let discovered = detector.detect(&test_file).unwrap();

    assert_eq!(
        discovered.root_path.file_name().unwrap(),
        repo_path.file_name().unwrap()
    );
}

#[test]
fn test_detect_non_repository_fails() {
    let temp_dir = TempDir::new().unwrap();
    let detector = GitDetector::new();

    let res = detector.detect(temp_dir.path());
    assert!(res.is_err());
}
