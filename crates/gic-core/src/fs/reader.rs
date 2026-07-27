use crate::error::GicError;
use crate::fs::document::{Document, DocumentContent, FileMetadata};
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::Path;

/// Threshold for large file streaming strategy (Default: 10 MB).
pub const DEFAULT_LARGE_FILE_THRESHOLD_BYTES: u64 = 10 * 1024 * 1024;

/// UTF-8 Byte Order Mark (BOM): 0xEF, 0xBB, 0xBF
const UTF8_BOM: &[u8] = &[0xEF, 0xBB, 0xBF];

/// File Reader component responsible for file ingestion, UTF-8 validation,
/// BOM handling, and large file line-chunking.
pub struct FileReader;

impl FileReader {
    /// Reads a file from disk into a domain `Document`.
    pub fn read_file<P: AsRef<Path>>(path: P) -> Result<Document, GicError> {
        Self::read_file_with_threshold(path, DEFAULT_LARGE_FILE_THRESHOLD_BYTES)
    }

    /// Reads a file from disk with a specified large file threshold (in bytes).
    pub fn read_file_with_threshold<P: AsRef<Path>>(
        path: P,
        large_file_threshold_bytes: u64,
    ) -> Result<Document, GicError> {
        let target_path = path.as_ref();

        // 1. Open File handle
        let mut file = File::open(target_path)
            .map_err(|e| GicError::from_io_error(target_path.to_path_buf(), e))?;

        // 2. Extract file metadata
        let fs_metadata = file
            .metadata()
            .map_err(|e| GicError::from_io_error(target_path.to_path_buf(), e))?;
        let size_bytes = fs_metadata.len();
        let is_large = size_bytes > large_file_threshold_bytes;

        if is_large {
            Self::read_large_file(target_path, file, size_bytes)
        } else {
            Self::read_standard_file(target_path, &mut file, size_bytes)
        }
    }

    /// Reads standard-sized file entirely into memory with UTF-8 and BOM validation.
    fn read_standard_file(
        path: &Path,
        file: &mut File,
        size_bytes: u64,
    ) -> Result<Document, GicError> {
        let mut raw_bytes = Vec::with_capacity(size_bytes as usize);
        file.read_to_end(&mut raw_bytes)
            .map_err(|e| GicError::from_io_error(path.to_path_buf(), e))?;

        let (has_bom, slice_to_decode) = if raw_bytes.starts_with(UTF8_BOM) {
            (true, &raw_bytes[3..])
        } else {
            (false, raw_bytes.as_slice())
        };

        let utf8_str =
            std::str::from_utf8(slice_to_decode).map_err(|utf8_err| GicError::InvalidUtf8 {
                path: path.to_path_buf(),
                position: Some(utf8_err.valid_up_to()),
                message: format!(
                    "Invalid UTF-8 sequence at byte offset {}",
                    utf8_err.valid_up_to()
                ),
            })?;

        let line_count = if utf8_str.is_empty() {
            0
        } else {
            utf8_str.lines().count()
        };

        let metadata = FileMetadata {
            size_bytes,
            line_count,
            has_bom,
            is_large: false,
            encoding: "UTF-8".to_string(),
        };

        let content = DocumentContent::Standard(utf8_str.to_string());
        Ok(Document::new(Some(path.to_path_buf()), content, metadata))
    }

    /// Reads large file via chunked line-buffered scanning to maintain safe memory footprint.
    fn read_large_file(path: &Path, file: File, size_bytes: u64) -> Result<Document, GicError> {
        let mut reader = BufReader::new(file);
        let mut lines = Vec::new();
        let mut line_buf = String::new();
        let mut total_lines = 0;
        let mut has_bom = false;
        let mut is_first_line = true;

        loop {
            line_buf.clear();
            let bytes_read = reader
                .read_line(&mut line_buf)
                .map_err(|e| GicError::from_io_error(path.to_path_buf(), e))?;

            if bytes_read == 0 {
                break; // EOF
            }

            // Strip trailing \r\n or \n for line storage
            let mut processed = line_buf.as_str();
            if is_first_line && processed.starts_with('\u{FEFF}') {
                has_bom = true;
                processed = &processed[3..];
                is_first_line = false;
            }

            let clean_line = processed.trim_end_matches(['\r', '\n']);
            lines.push(clean_line.to_string());
            total_lines += 1;
        }

        let metadata = FileMetadata {
            size_bytes,
            line_count: total_lines,
            has_bom,
            is_large: true,
            encoding: "UTF-8".to_string(),
        };

        let content = DocumentContent::LargeFile(lines);
        Ok(Document::new(Some(path.to_path_buf()), content, metadata))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_read_valid_utf8_file() {
        let mut temp = NamedTempFile::new().unwrap();
        writeln!(temp, "Hello GIC\nLine 2").unwrap();

        let doc = FileReader::read_file(temp.path()).unwrap();
        assert_eq!(doc.content.as_str_content().trim_end(), "Hello GIC\nLine 2");
        assert_eq!(doc.metadata.line_count, 2);
        assert!(!doc.metadata.has_bom);
        assert!(!doc.metadata.is_large);
    }

    #[test]
    fn test_read_utf8_bom_file() {
        let mut temp = NamedTempFile::new().unwrap();
        temp.write_all(UTF8_BOM).unwrap();
        temp.write_all(b"Header line\nData line").unwrap();

        let doc = FileReader::read_file(temp.path()).unwrap();
        assert!(doc.metadata.has_bom);
        assert_eq!(doc.content.as_str_content(), "Header line\nData line");
        assert_eq!(doc.metadata.line_count, 2);
    }

    #[test]
    fn test_read_invalid_utf8_file() {
        let mut temp = NamedTempFile::new().unwrap();
        // Write invalid UTF-8 bytes (e.g. 0xFF, 0xFE)
        temp.write_all(&[0x68, 0x65, 0x6C, 0x6C, 0x6F, 0xFF, 0xFE])
            .unwrap();

        let result = FileReader::read_file(temp.path());
        assert!(result.is_err());
        if let Err(GicError::InvalidUtf8 { position, .. }) = result {
            assert_eq!(position, Some(5));
        } else {
            panic!("Expected InvalidUtf8 error");
        }
    }

    #[test]
    fn test_read_large_file_threshold() {
        let mut temp = NamedTempFile::new().unwrap();
        writeln!(temp, "Line 1\nLine 2\nLine 3").unwrap();

        // Threshold = 5 bytes -> triggers large file path
        let doc = FileReader::read_file_with_threshold(temp.path(), 5).unwrap();
        assert!(doc.metadata.is_large);
        assert_eq!(doc.metadata.line_count, 3);
        assert_eq!(doc.content.as_str_content(), "Line 1\nLine 2\nLine 3");
    }

    #[test]
    fn test_read_nonexistent_file() {
        let path = Path::new("non_existent_file_path_12345.txt");
        let result = FileReader::read_file(path);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), GicError::FileNotFound(_)));
    }
}
