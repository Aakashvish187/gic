//! File System Layer for GIC (General Infrastructure Console).
//!
//! Provides domain entities and services for reading, writing, UTF-8 validation,
//! BOM handling, large file streaming, and recent files MRU history management.

pub mod document;
pub mod manager;
pub mod reader;
pub mod recent;

pub use document::{Document, DocumentContent, FileMetadata};
pub use manager::FileSystemManager;
pub use reader::FileReader;
pub use recent::{RecentFileEntry, RecentFilesManager};

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_fs_module_integration() {
        let mut fs_mgr = FileSystemManager::new();

        let mut temp = NamedTempFile::new().unwrap();
        writeln!(temp, "Integration Test File").unwrap();
        let path = temp.path();

        let mut doc = fs_mgr.open_file(path).unwrap();
        assert_eq!(
            doc.content.as_str_content().trim_end(),
            "Integration Test File"
        );
        assert_eq!(fs_mgr.recent_files().len(), 1);

        doc.content = DocumentContent::Standard("Modified Integration Data".into());
        doc.mark_modified();

        fs_mgr.save_file(&mut doc).unwrap();
        assert!(!doc.is_modified);

        let reloaded = fs_mgr.open_file(path).unwrap();
        assert_eq!(
            reloaded.content.as_str_content(),
            "Modified Integration Data"
        );
    }
}
