use std::path::PathBuf;
use thiserror::Error;

/// Central domain error enum for GIC application.
#[derive(Debug, Error)]
pub enum GicError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Logging initialization error: {0}")]
    Logging(String),

    #[error("Terminal I/O error: {0}")]
    Terminal(String),

    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),

    #[error("File not found: {0}")]
    FileNotFound(PathBuf),

    #[error("Permission denied: {0}")]
    PermissionDenied(PathBuf),

    #[error("Invalid UTF-8 in file '{path}': {message}")]
    InvalidUtf8 {
        path: PathBuf,
        position: Option<usize>,
        message: String,
    },

    #[error("File too large ({size} bytes exceeds max {max_size} bytes): {path}")]
    FileTooLarge {
        path: PathBuf,
        size: u64,
        max_size: u64,
    },

    #[error("No file path specified for operation")]
    NoPathSpecified,

    #[error("File system I/O error on '{path}': {source}")]
    FsIo {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error("Search engine error: {0}")]
    Search(#[from] crate::search::SearchError),
}

impl GicError {
    /// Maps a std::io::Error and PathBuf into a domain GicError based on std::io::ErrorKind.
    pub fn from_io_error(path: PathBuf, err: std::io::Error) -> Self {
        match err.kind() {
            std::io::ErrorKind::NotFound => GicError::FileNotFound(path),
            std::io::ErrorKind::PermissionDenied => GicError::PermissionDenied(path),
            _ => GicError::FsIo { path, source: err },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_error_formatting() {
        let err = GicError::Config("file not found".into());
        assert_eq!(format!("{err}"), "Configuration error: file not found");
    }

    #[test]
    fn test_logging_error_formatting() {
        let err = GicError::Logging("permission denied".into());
        assert_eq!(
            format!("{err}"),
            "Logging initialization error: permission denied"
        );
    }

    #[test]
    fn test_terminal_error_formatting() {
        let err = GicError::Terminal("failed raw mode".into());
        assert_eq!(format!("{err}"), "Terminal I/O error: failed raw mode");
    }

    #[test]
    fn test_io_error_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file missing");
        let gic_err: GicError = io_err.into();
        assert!(matches!(gic_err, GicError::Io(_)));
    }

    #[test]
    fn test_fs_error_formatting() {
        let path = PathBuf::from("missing.txt");
        let fnf = GicError::FileNotFound(path.clone());
        assert_eq!(format!("{fnf}"), "File not found: missing.txt");

        let perm = GicError::PermissionDenied(path.clone());
        assert_eq!(format!("{perm}"), "Permission denied: missing.txt");

        let invalid_utf8 = GicError::InvalidUtf8 {
            path: path.clone(),
            position: Some(10),
            message: "invalid byte sequence".into(),
        };
        assert_eq!(
            format!("{invalid_utf8}"),
            "Invalid UTF-8 in file 'missing.txt': invalid byte sequence"
        );

        let too_large = GicError::FileTooLarge {
            path: path.clone(),
            size: 2000,
            max_size: 1000,
        };
        assert_eq!(
            format!("{too_large}"),
            "File too large (2000 bytes exceeds max 1000 bytes): missing.txt"
        );

        let no_path = GicError::NoPathSpecified;
        assert_eq!(format!("{no_path}"), "No file path specified for operation");
    }

    #[test]
    fn test_from_io_error_mapping() {
        let path = PathBuf::from("dummy.txt");
        let io_not_found = std::io::Error::new(std::io::ErrorKind::NotFound, "not found");
        let err1 = GicError::from_io_error(path.clone(), io_not_found);
        assert!(matches!(err1, GicError::FileNotFound(_)));

        let io_perm = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "denied");
        let err2 = GicError::from_io_error(path.clone(), io_perm);
        assert!(matches!(err2, GicError::PermissionDenied(_)));
    }
}
