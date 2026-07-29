use std::path::{Path, PathBuf};

/// Detects the project root by walking up the directory tree
/// looking for common project indicators.
pub fn detect_project_root(start_path: &Path) -> Option<PathBuf> {
    let mut current = if start_path.is_file() {
        start_path.parent().map(Path::to_path_buf)
    } else {
        Some(start_path.to_path_buf())
    };

    let markers = [
        ".git",
        "Cargo.toml",
        "package.json",
        "docker-compose.yml",
        "Makefile",
    ];

    while let Some(path) = current {
        for marker in &markers {
            if path.join(marker).exists() {
                return Some(path);
            }
        }
        current = path.parent().map(|p| p.to_path_buf());
    }

    // Fallback: if no markers found, use the directory of the file or current dir
    if start_path.is_file() {
        start_path.parent().map(Path::to_path_buf)
    } else {
        Some(start_path.to_path_buf())
    }
}
