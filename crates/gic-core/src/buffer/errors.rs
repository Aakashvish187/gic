use thiserror::Error;

/// Domain error enum for Text Buffer operations in GIC.
#[derive(Debug, Error, PartialEq, Eq, Clone)]
pub enum BufferError {
    /// Specified cursor position is out of buffer bounds.
    #[error("Invalid cursor position: row {row}, col {col}")]
    InvalidPosition { row: usize, col: usize },

    /// Operation requested on an empty buffer when content was expected.
    #[error("Buffer is empty")]
    EmptyBuffer,

    /// Invalid selection range requested or no active selection.
    #[error("No active or valid selection")]
    InvalidSelection,

    /// Attempted to paste when internal clipboard is empty.
    #[error("Internal clipboard is empty")]
    ClipboardEmpty,

    /// Attempted undo or redo when history stack is empty.
    #[error("Undo/Redo history stack is empty")]
    HistoryEmpty,

    /// General buffer operation failure with custom message.
    #[error("Buffer operation error: {0}")]
    OperationFailed(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer_error_formatting() {
        let invalid_pos = BufferError::InvalidPosition { row: 10, col: 5 };
        assert_eq!(
            format!("{invalid_pos}"),
            "Invalid cursor position: row 10, col 5"
        );

        let empty_buf = BufferError::EmptyBuffer;
        assert_eq!(format!("{empty_buf}"), "Buffer is empty");

        let no_sel = BufferError::InvalidSelection;
        assert_eq!(format!("{no_sel}"), "No active or valid selection");

        let empty_clip = BufferError::ClipboardEmpty;
        assert_eq!(format!("{empty_clip}"), "Internal clipboard is empty");

        let empty_hist = BufferError::HistoryEmpty;
        assert_eq!(format!("{empty_hist}"), "Undo/Redo history stack is empty");

        let op_fail = BufferError::OperationFailed("out of range".into());
        assert_eq!(format!("{op_fail}"), "Buffer operation error: out of range");
    }
}
