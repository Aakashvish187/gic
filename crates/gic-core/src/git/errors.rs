//! Error types for the Git Awareness Engine.

use std::path::PathBuf;
use thiserror::Error;

/// Errors produced by the Git Awareness Engine.
#[derive(Debug, Error)]
pub enum GitError {
    #[error("Repository not found at or above path: {0}")]
    RepositoryNotFound(PathBuf),

    #[error("Invalid repository at path {0}: {1}")]
    InvalidRepository(PathBuf, String),

    #[error("Failed to read branch information: {0}")]
    BranchError(String),

    #[error("Failed to calculate diff: {0}")]
    DiffError(String),

    #[error("Git2 error: {0}")]
    Git2Error(#[from] git2::Error),

    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Corrupt HEAD reference in repository: {0}")]
    CorruptHead(String),

    #[error("Cache error: {0}")]
    CacheError(String),

    #[error("Ignore pattern error: {0}")]
    IgnoreError(String),
}

/// Convenience Result type for Git operations.
pub type GitResult<T> = Result<T, GitError>;
