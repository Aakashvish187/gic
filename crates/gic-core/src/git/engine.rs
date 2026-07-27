//! Unified Façade Engine for Git Awareness in GIC.

use crate::git::blame::{BlameEngine, BlameHunk};
use crate::git::branch::{BranchEngine, GitBranch};
use crate::git::cache::GitCache;
use crate::git::decorations::{EditorDecorations, GutterDecoration, StatusBarGitInfo};
use crate::git::detector::{DiscoveredRepo, GitDetector};
use crate::git::diagnostics::GitDiagnostics;
use crate::git::diff::{DiffEngine, DiffOptions, FileDiff};
use crate::git::errors::GitResult;
use crate::git::history::{GitCommit, HistoryEngine};
use crate::git::ignore::GitIgnoreEngine;
use crate::git::logger::GitLogger;
use crate::git::metrics::GitMetrics;
use crate::git::repository::GitRepository;
use crate::git::status::{FileStatus, RepositoryStatus, StatusEngine};
use std::collections::HashMap;
use std::path::Path;

/// Main façade orchestrating all read-only Git awareness features.
#[derive(Debug, Clone, Default)]
pub struct GitEngine {
    detector: GitDetector,
    status_engine: StatusEngine,
    diff_engine: DiffEngine,
    branch_engine: BranchEngine,
    history_engine: HistoryEngine,
    blame_engine: BlameEngine,
    decorations: EditorDecorations,
    diagnostics: GitDiagnostics,
    cache: GitCache,
    logger: GitLogger,
    pub metrics: GitMetrics,
}

impl GitEngine {
    pub fn new() -> Self {
        Self::default()
    }

    /// Detects if a directory or file is inside a Git repository.
    pub fn detect_repository<P: AsRef<Path>>(&self, path: P) -> GitResult<DiscoveredRepo> {
        self.metrics.inc_repo_detections();
        let res = self.detector.detect(path)?;
        self.logger.log_repo_detected(&res.root_path, res.is_bare);
        Ok(res)
    }

    /// Opens a `GitRepository` handle for a given path.
    pub fn open_repository<P: AsRef<Path>>(&self, path: P) -> GitResult<GitRepository> {
        GitRepository::open_from_path(path)
    }

    /// Retrieves current branch details for a repository.
    pub fn current_branch(&self, repo: &GitRepository) -> GitResult<GitBranch> {
        self.branch_engine.current_branch(repo)
    }

    /// Computes full repository status.
    pub fn repository_status(&self, repo: &GitRepository) -> GitResult<RepositoryStatus> {
        self.metrics.inc_status_scans();
        let status = self.status_engine.repository_status(repo)?;
        self.logger.log_status_scanned(
            &repo.root_path,
            status.modified_count,
            status.entries.len(),
        );
        Ok(status)
    }

    /// Computes single file status.
    pub fn file_status<P: AsRef<Path>>(
        &self,
        repo: &GitRepository,
        relative_path: P,
    ) -> GitResult<FileStatus> {
        self.status_engine.file_status(repo, relative_path)
    }

    /// Computes buffer vs HEAD commit line-by-line diff.
    pub fn compute_buffer_diff<P: AsRef<Path>>(
        &self,
        repo: &GitRepository,
        relative_path: P,
        buffer_content: &str,
        options: Option<&DiffOptions>,
    ) -> GitResult<FileDiff> {
        self.metrics.inc_diff_computations();
        let default_opts = DiffOptions::default();
        let opts = options.unwrap_or(&default_opts);
        let diff = self.diff_engine.compute_buffer_diff(
            repo,
            relative_path.as_ref(),
            buffer_content,
            opts,
        )?;
        self.logger.log_diff_calculated(
            relative_path.as_ref(),
            diff.added_count,
            diff.deleted_count,
            diff.modified_count,
        );
        Ok(diff)
    }

    /// Provides line gutter decorations for the active buffer.
    pub fn gutter_decorations(&self, diff: &FileDiff) -> HashMap<usize, GutterDecoration> {
        self.decorations.line_gutter_decorations(diff)
    }

    /// Assembles status bar Git metadata payload for UI rendering.
    pub fn status_bar_info(
        &self,
        repo: &GitRepository,
        file_status: FileStatus,
        file_diff: Option<&FileDiff>,
    ) -> GitResult<StatusBarGitInfo> {
        let repo_name = repo
            .root_path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let branch = self.current_branch(repo)?;
        let status = self.repository_status(repo)?;
        Ok(self.decorations.build_status_bar_info(
            repo_name,
            &branch,
            &status,
            file_status,
            file_diff,
        ))
    }

    /// Fetches recent commit history.
    pub fn recent_commits<P: AsRef<Path>>(
        &self,
        repo: &GitRepository,
        relative_path: Option<P>,
        limit: usize,
    ) -> GitResult<Vec<GitCommit>> {
        self.history_engine
            .recent_commits(repo, relative_path, limit)
    }

    /// Fetches line blame info for hover tooltips.
    pub fn blame_line<P: AsRef<Path>>(
        &self,
        repo: &GitRepository,
        relative_path: P,
        line_number: usize,
    ) -> GitResult<Option<BlameHunk>> {
        self.blame_engine
            .blame_line(repo, relative_path, line_number)
    }

    /// Checks if a file is ignored by .gitignore rules.
    pub fn is_ignored(&self, repo: &GitRepository, file_path: &Path) -> bool {
        if let Ok(ignore_engine) = GitIgnoreEngine::new(repo) {
            ignore_engine.is_ignored(file_path)
        } else {
            false
        }
    }

    /// Access to diagnostics generator.
    pub fn diagnostics(&self) -> &GitDiagnostics {
        &self.diagnostics
    }

    /// Access to thread-safe Git cache.
    pub fn cache(&self) -> &GitCache {
        &self.cache
    }
}
