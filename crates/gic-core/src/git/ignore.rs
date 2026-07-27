//! Git ignore pattern engine evaluating root and nested `.gitignore` files.

use crate::git::errors::GitResult;
use crate::git::repository::GitRepository;
use ignore::gitignore::GitignoreBuilder;
use std::path::{Path, PathBuf};

/// Engine for checking whether files are ignored by `.gitignore` rules.
pub struct GitIgnoreEngine {
    repo_root: PathBuf,
    builder: GitignoreBuilder,
}

impl std::fmt::Debug for GitIgnoreEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GitIgnoreEngine")
            .field("repo_root", &self.repo_root)
            .finish()
    }
}

impl GitIgnoreEngine {
    /// Constructs a `GitIgnoreEngine` for the given repository.
    pub fn new(repo: &GitRepository) -> GitResult<Self> {
        let root = &repo.root_path;
        let mut builder = GitignoreBuilder::new(root);

        // Add root .gitignore if present
        let root_gitignore = root.join(".gitignore");
        if root_gitignore.is_file() {
            builder.add(&root_gitignore);
        }

        // Add nested .gitignore files
        if let Ok(entries) = std::fs::read_dir(root) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let nested = path.join(".gitignore");
                    if nested.is_file() {
                        builder.add(&nested);
                    }
                }
            }
        }

        Ok(Self {
            repo_root: root.clone(),
            builder,
        })
    }

    /// Checks if a file path relative to repo root or absolute path is ignored.
    pub fn is_ignored<P: AsRef<Path>>(&self, path: P) -> bool {
        let path = path.as_ref();
        let abs_path = if path.is_absolute() {
            path.to_path_buf()
        } else {
            self.repo_root.join(path)
        };

        if let Ok(gitignore) = self.builder.build() {
            let is_dir = abs_path.is_dir();
            gitignore.matched(&abs_path, is_dir).is_ignore()
        } else {
            false
        }
    }
}
