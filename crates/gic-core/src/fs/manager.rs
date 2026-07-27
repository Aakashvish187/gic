use crate::error::GicError;
use crate::fs::document::Document;
use crate::fs::reader::FileReader;
use crate::fs::recent::RecentFilesManager;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

/// High-level domain service manager for File System operations in GIC.
/// Coordinates file opening, saving, "Save As", recent files MRU history,
/// atomic disk writes, UTF-8/BOM preservation, and error handling.
pub struct FileSystemManager {
    recent_files: RecentFilesManager,
    large_file_threshold_bytes: u64,
}

impl Default for FileSystemManager {
    fn default() -> Self {
        Self::new()
    }
}

impl FileSystemManager {
    /// Creates a new `FileSystemManager` with default settings.
    pub fn new() -> Self {
        Self {
            recent_files: RecentFilesManager::default(),
            large_file_threshold_bytes: crate::fs::reader::DEFAULT_LARGE_FILE_THRESHOLD_BYTES,
        }
    }

    /// Creates a `FileSystemManager` with custom recent files capacity and large file threshold.
    pub fn with_settings(recent_files_capacity: usize, large_file_threshold_bytes: u64) -> Self {
        Self {
            recent_files: RecentFilesManager::new(recent_files_capacity),
            large_file_threshold_bytes,
        }
    }

    /// Access reference to Recent Files Manager.
    pub fn recent_files(&self) -> &RecentFilesManager {
        &self.recent_files
    }

    /// Access mutable reference to Recent Files Manager.
    pub fn recent_files_mut(&mut self) -> &mut RecentFilesManager {
        &mut self.recent_files
    }

    /// Opens a file from disk into a `Document` and registers it in Recent Files history.
    pub fn open_file<P: AsRef<Path>>(&mut self, path: P) -> Result<Document, GicError> {
        let target_path = path.as_ref();
        let doc =
            FileReader::read_file_with_threshold(target_path, self.large_file_threshold_bytes)?;

        // Register in Recent Files
        self.recent_files
            .add_path(target_path, doc.metadata.size_bytes);

        Ok(doc)
    }

    /// Saves the current document back to its assigned path.
    /// Returns `GicError::NoPathSpecified` if document has no established path.
    pub fn save_file(&mut self, doc: &mut Document) -> Result<(), GicError> {
        let target_path = match &doc.path {
            Some(p) => p.clone(),
            None => return Err(GicError::NoPathSpecified),
        };

        self.write_document_to_disk(&target_path, doc)?;
        doc.mark_saved();
        self.recent_files
            .add_path(&target_path, doc.metadata.size_bytes);
        Ok(())
    }

    /// Saves document to a newly specified file path ("Save As").
    /// Updates document path, resets modified flag, and updates Recent Files.
    pub fn save_file_as<P: AsRef<Path>>(
        &mut self,
        doc: &mut Document,
        new_path: P,
    ) -> Result<(), GicError> {
        let target_path = new_path.as_ref().to_path_buf();
        self.write_document_to_disk(&target_path, doc)?;

        doc.set_path(&target_path);
        doc.mark_saved();
        self.recent_files
            .add_path(&target_path, doc.metadata.size_bytes);
        Ok(())
    }

    /// Helper to write document content atomically to disk.
    fn write_document_to_disk(
        &self,
        target_path: &PathBuf,
        doc: &mut Document,
    ) -> Result<(), GicError> {
        let parent_dir = target_path.parent().unwrap_or_else(|| Path::new("."));

        // Create temporary file path in same parent directory for atomic swap
        let file_stem = target_path
            .file_name()
            .map(|n| n.to_string_lossy())
            .unwrap_or_else(|| "gic_tmp".into());
        let tmp_file_name = format!(".gic_tmp_{}_{}.tmp", file_stem, std::process::id());
        let tmp_path = parent_dir.join(tmp_file_name);

        // 1. Create and write to temp file
        let mut tmp_file =
            File::create(&tmp_path).map_err(|e| GicError::from_io_error(target_path.clone(), e))?;

        // Write UTF-8 BOM if present in metadata
        if doc.metadata.has_bom {
            tmp_file
                .write_all(&[0xEF, 0xBB, 0xBF])
                .map_err(|e| GicError::from_io_error(target_path.clone(), e))?;
        }

        // Write text content
        let content_str = doc.content.as_str_content();
        tmp_file
            .write_all(content_str.as_bytes())
            .map_err(|e| GicError::from_io_error(target_path.clone(), e))?;

        tmp_file
            .flush()
            .map_err(|e| GicError::from_io_error(target_path.clone(), e))?;
        drop(tmp_file);

        // 2. Atomic Rename
        if let Err(err) = std::fs::rename(&tmp_path, target_path) {
            // Clean up temporary file on rename error
            let _ = std::fs::remove_file(&tmp_path);
            return Err(GicError::from_io_error(target_path.clone(), err));
        }

        // 3. Update Document Metadata
        let written_bytes = content_str.len() as u64 + if doc.metadata.has_bom { 3 } else { 0 };
        doc.metadata.size_bytes = written_bytes;
        doc.metadata.line_count = if content_str.is_empty() {
            0
        } else {
            content_str.lines().count()
        };
        doc.metadata.is_large = written_bytes > self.large_file_threshold_bytes;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fs::document::DocumentContent;
    use tempfile::NamedTempFile;

    #[test]
    fn test_open_save_and_recent_files_workflow() {
        let mut fs_mgr = FileSystemManager::new();

        // 1. Create temp file
        let mut temp = NamedTempFile::new().unwrap();
        writeln!(temp, "Hello GIC File System").unwrap();
        let path = temp.path().to_path_buf();

        // 2. Open File
        let mut doc = fs_mgr.open_file(&path).unwrap();
        assert_eq!(
            doc.content.as_str_content().trim_end(),
            "Hello GIC File System"
        );
        assert_eq!(fs_mgr.recent_files().len(), 1);

        // 3. Modify & Save File
        doc.content = DocumentContent::Standard("Updated Content\nLine 2".into());
        doc.mark_modified();
        assert!(doc.is_modified);

        fs_mgr.save_file(&mut doc).unwrap();
        assert!(!doc.is_modified);

        // Re-read file to verify atomic disk write
        let doc_reopened = fs_mgr.open_file(&path).unwrap();
        assert_eq!(
            doc_reopened.content.as_str_content(),
            "Updated Content\nLine 2"
        );
    }

    #[test]
    fn test_save_no_path_returns_error() {
        let mut fs_mgr = FileSystemManager::new();
        let mut doc = Document::new_empty();

        let result = fs_mgr.save_file(&mut doc);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), GicError::NoPathSpecified));
    }

    #[test]
    fn test_save_as_workflow() {
        let mut fs_mgr = FileSystemManager::new();
        let mut doc = Document::new_empty();
        doc.content = DocumentContent::Standard("Save As Data".into());
        doc.mark_modified();

        let temp_dest = NamedTempFile::new().unwrap();
        let dest_path = temp_dest.path().to_path_buf();

        // Perform Save As
        fs_mgr.save_file_as(&mut doc, &dest_path).unwrap();

        assert_eq!(doc.path, Some(dest_path.clone()));
        assert!(!doc.is_modified);
        assert_eq!(fs_mgr.recent_files().len(), 1);

        // Verify content written to dest_path
        let reopened = fs_mgr.open_file(&dest_path).unwrap();
        assert_eq!(reopened.content.as_str_content(), "Save As Data");
    }

    #[test]
    fn test_bom_preservation_on_save() {
        let mut fs_mgr = FileSystemManager::new();
        let temp = NamedTempFile::new().unwrap();
        let path = temp.path().to_path_buf();

        let mut doc = Document::new_empty();
        doc.set_path(&path);
        doc.metadata.has_bom = true;
        doc.content = DocumentContent::Standard("BOM Content".into());

        fs_mgr.save_file(&mut doc).unwrap();

        // Re-open and verify BOM was detected and preserved
        let reopened = fs_mgr.open_file(&path).unwrap();
        assert!(reopened.metadata.has_bom);
        assert_eq!(reopened.content.as_str_content(), "BOM Content");
    }
}
