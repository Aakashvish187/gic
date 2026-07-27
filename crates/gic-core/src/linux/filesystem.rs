//! Filesystem and Mount Point Analysis.
//!
//! Validates paths, `/etc/fstab` configuration, disk usage,
//! and symbolic/hard link management in shell scripts.

use crate::linux::errors::LinuxResult;
use crate::yaml::parser::{Position, Span};

/// Filesystem validation diagnostic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilesystemDiagnostic {
    pub rule_id: String,
    pub message: String,
    pub span: Span,
    pub is_error: bool,
}

/// Filesystem syntax and rule analyzer.
#[derive(Debug, Clone, Default)]
pub struct FilesystemAnalyzer;

impl FilesystemAnalyzer {
    pub fn new() -> Self {
        Self
    }

    /// Analyzes `/etc/fstab` configuration.
    pub fn analyze_fstab(&self, source: &str) -> LinuxResult<Vec<FilesystemDiagnostic>> {
        let mut diagnostics = Vec::new();

        for (idx, line) in source.lines().enumerate() {
            let line_num = idx + 1;
            let trimmed = line.trim();

            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if parts.len() < 4 {
                let pos_start = Position::new(line_num, 1, 0);
                let pos_end = Position::new(line_num, line.len().max(1), 0);
                diagnostics.push(FilesystemDiagnostic {
                    rule_id: "lin-fs-fstab".to_string(),
                    message: "Invalid fstab entry: requires at least 4 fields (fs_spec, fs_file, fs_vfstype, fs_mntops)".to_string(),
                    span: Span::new(pos_start, pos_end),
                    is_error: true,
                });
            }
        }

        Ok(diagnostics)
    }
}
