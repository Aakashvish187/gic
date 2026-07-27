//! Git diagnostic generator converting repository status into GIC diagnostics.

use crate::diagnostics::{Diagnostic, DiagnosticLevel, DiagnosticPosition, DiagnosticRange};
use crate::git::branch::{BranchKind, GitBranch};
use crate::git::status::{FileStatus, RepositoryStatus};
use crate::parser::LanguageId;
use std::path::Path;

/// Engine generating diagnostics for Git repository awareness.
#[derive(Debug, Clone, Default)]
pub struct GitDiagnostics;

impl GitDiagnostics {
    pub fn new() -> Self {
        Self
    }

    /// Generates repository-level diagnostics (e.g. detached HEAD, merge conflicts).
    pub fn generate_repo_diagnostics(
        &self,
        branch: &GitBranch,
        status: &RepositoryStatus,
    ) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        let default_range =
            DiagnosticRange::new(DiagnosticPosition::zero(), DiagnosticPosition::zero());

        if branch.kind == BranchKind::Detached {
            diagnostics.push(Diagnostic::new(
                DiagnosticLevel::Warning,
                format!(
                    "Repository is in a detached HEAD state at commit {}.",
                    branch.short_oid
                ),
                default_range,
                "GIT001-DetachedHEAD",
                LanguageId::PlainText,
            ));
        }

        if status.conflicted_count > 0 {
            diagnostics.push(Diagnostic::new(
                DiagnosticLevel::Error,
                format!(
                    "Repository has {} unresolved merge conflict(s).",
                    status.conflicted_count
                ),
                default_range,
                "GIT002-UnresolvedMergeConflicts",
                LanguageId::PlainText,
            ));
        }

        diagnostics
    }

    /// Generates file-specific Git diagnostics.
    pub fn generate_file_diagnostics<P: AsRef<Path>>(
        &self,
        path: P,
        status: FileStatus,
    ) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        let path = path.as_ref();
        let default_range =
            DiagnosticRange::new(DiagnosticPosition::zero(), DiagnosticPosition::zero());

        if status == FileStatus::Conflicted {
            diagnostics.push(Diagnostic::new(
                DiagnosticLevel::Error,
                format!("File {} contains active merge conflicts.", path.display()),
                default_range,
                "GIT003-FileConflict",
                LanguageId::PlainText,
            ));
        }

        diagnostics
    }
}
