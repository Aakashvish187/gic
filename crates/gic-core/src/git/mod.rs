//! Git Awareness Engine – public API

pub mod blame;
pub mod branch;
pub mod cache;
pub mod decorations;
pub mod detector;
pub mod diagnostics;
pub mod diff;
pub mod engine;
pub mod errors;
pub mod history;
pub mod ignore;
pub mod logger;
pub mod metrics;
pub mod repository;
pub mod status;

#[cfg(test)]
pub mod tests;

pub use blame::{BlameEngine, BlameHunk};
pub use branch::{BranchEngine, BranchKind, GitBranch};
pub use cache::{CachedRepoData, GitCache};
pub use decorations::{EditorDecorations, GutterDecoration, LineDecoration, StatusBarGitInfo};
pub use detector::{DiscoveredRepo, GitDetector};
pub use diagnostics::GitDiagnostics;
pub use diff::{DiffEngine, DiffHunk, DiffLine, DiffOptions, FileDiff, LineChangeKind};
pub use engine::GitEngine;
pub use errors::{GitError, GitResult};
pub use history::{GitCommit, HistoryEngine};
pub use ignore::GitIgnoreEngine;
pub use logger::GitLogger;
pub use metrics::GitMetrics;
pub use repository::GitRepository;
pub use status::{FileStatus, FileStatusEntry, RepositoryStatus, StatusEngine};
