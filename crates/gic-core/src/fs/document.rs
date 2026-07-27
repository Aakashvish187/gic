use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Telemetry and attributes of a file managed by GIC.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileMetadata {
    pub size_bytes: u64,
    pub line_count: usize,
    pub has_bom: bool,
    pub is_large: bool,
    pub encoding: String,
}

impl Default for FileMetadata {
    fn default() -> Self {
        Self {
            size_bytes: 0,
            line_count: 0,
            has_bom: false,
            is_large: false,
            encoding: "UTF-8".to_string(),
        }
    }
}

/// Content representation of a document.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DocumentContent {
    /// In-memory UTF-8 text string for standard-sized files.
    Standard(String),
    /// Line-chunked storage for large files exceeding threshold.
    LargeFile(Vec<String>),
}

impl Default for DocumentContent {
    fn default() -> Self {
        DocumentContent::Standard(String::new())
    }
}

impl DocumentContent {
    /// Consolidates content into a single String representation.
    pub fn as_str_content(&self) -> String {
        match self {
            DocumentContent::Standard(s) => s.clone(),
            DocumentContent::LargeFile(lines) => lines.join("\n"),
        }
    }

    /// Total byte length of document content.
    pub fn byte_len(&self) -> usize {
        match self {
            DocumentContent::Standard(s) => s.len(),
            DocumentContent::LargeFile(lines) => {
                lines.iter().map(|l| l.len()).sum::<usize>() + lines.len().saturating_sub(1)
            }
        }
    }

    /// Returns true if document content is empty.
    pub fn is_empty(&self) -> bool {
        match self {
            DocumentContent::Standard(s) => s.is_empty(),
            DocumentContent::LargeFile(lines) => lines.is_empty(),
        }
    }
}

/// Core domain object representing an open file or newly created document in GIC.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Document {
    pub path: Option<PathBuf>,
    pub content: DocumentContent,
    pub metadata: FileMetadata,
    pub is_modified: bool,
    pub is_read_only: bool,
}

impl Default for Document {
    fn default() -> Self {
        Self::new_empty()
    }
}

impl Document {
    /// Creates an empty untitled document.
    pub fn new_empty() -> Self {
        Self {
            path: None,
            content: DocumentContent::Standard(String::new()),
            metadata: FileMetadata::default(),
            is_modified: false,
            is_read_only: false,
        }
    }

    /// Creates a document with specified path, content, and metadata.
    pub fn new(path: Option<PathBuf>, content: DocumentContent, metadata: FileMetadata) -> Self {
        Self {
            path,
            content,
            metadata,
            is_modified: false,
            is_read_only: false,
        }
    }

    /// Sets the document file path.
    pub fn set_path<P: AsRef<Path>>(&mut self, path: P) {
        self.path = Some(path.as_ref().to_path_buf());
    }

    /// Marks document as saved (resets `is_modified` flag).
    pub fn mark_saved(&mut self) {
        self.is_modified = false;
    }

    /// Marks document as modified.
    pub fn mark_modified(&mut self) {
        self.is_modified = true;
    }

    /// Returns human-readable file name if path is established.
    pub fn file_name(&self) -> Option<String> {
        self.path
            .as_ref()
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_empty_document() {
        let doc = Document::new_empty();
        assert!(doc.path.is_none());
        assert!(doc.content.is_empty());
        assert!(!doc.is_modified);
        assert!(!doc.is_read_only);
        assert_eq!(doc.metadata.encoding, "UTF-8");
    }

    #[test]
    fn test_document_path_and_file_name() {
        let mut doc = Document::new_empty();
        assert_eq!(doc.file_name(), None);

        doc.set_path("z:/test/example.txt");
        assert_eq!(doc.file_name(), Some("example.txt".to_string()));
    }

    #[test]
    fn test_document_modified_and_saved_flags() {
        let mut doc = Document::new_empty();
        assert!(!doc.is_modified);

        doc.mark_modified();
        assert!(doc.is_modified);

        doc.mark_saved();
        assert!(!doc.is_modified);
    }

    #[test]
    fn test_document_content_operations() {
        let std_content = DocumentContent::Standard("Hello World".to_string());
        assert_eq!(std_content.as_str_content(), "Hello World");
        assert_eq!(std_content.byte_len(), 11);
        assert!(!std_content.is_empty());

        let large_content = DocumentContent::LargeFile(vec!["Line 1".into(), "Line 2".into()]);
        assert_eq!(large_content.as_str_content(), "Line 1\nLine 2");
        assert_eq!(large_content.byte_len(), 13);
        assert!(!large_content.is_empty());
    }
}
