//! # Dirty File Indicator
//!
//! Tracks and displays the modified/saved state of the current document.
//! Provides display symbols and status text for integration with the
//! status bar and other UI elements.
//!
//! ## Design
//!
//! The dirty indicator reads directly from the source-of-truth state
//! (TextBuffer and Document), ensuring zero false positives. It does
//! not maintain its own shadow state.

/// Display symbols and text for the document modification state.
///
/// The dirty indicator is stateless — it computes display values from
/// the current modified/read-only state each time it's queried.
pub struct DirtyIndicator;

impl DirtyIndicator {
    /// Returns the appropriate indicator symbol for the current state.
    ///
    /// - `●` (filled circle) — Document has unsaved changes.
    /// - `✓` (check mark) — Document is saved (no modifications).
    /// - `🔒` (lock) — Document is read-only.
    pub fn symbol(is_modified: bool, is_read_only: bool) -> &'static str {
        if is_read_only {
            "🔒"
        } else if is_modified {
            "●"
        } else {
            "✓"
        }
    }

    /// Returns a descriptive status text for the current state.
    pub fn status_text(is_modified: bool, is_read_only: bool) -> &'static str {
        if is_read_only {
            "Read Only"
        } else if is_modified {
            "Modified"
        } else {
            "Saved"
        }
    }

    /// Returns a short label for tight-space displays.
    pub fn short_label(is_modified: bool, is_read_only: bool) -> &'static str {
        if is_read_only {
            "RO"
        } else if is_modified {
            "[+]"
        } else {
            ""
        }
    }

    /// Returns true if the document state should be highlighted
    /// (i.e., there are unsaved changes).
    pub fn should_highlight(is_modified: bool) -> bool {
        is_modified
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symbol_modified() {
        assert_eq!(DirtyIndicator::symbol(true, false), "●");
    }

    #[test]
    fn test_symbol_saved() {
        assert_eq!(DirtyIndicator::symbol(false, false), "✓");
    }

    #[test]
    fn test_symbol_read_only() {
        assert_eq!(DirtyIndicator::symbol(false, true), "🔒");
        assert_eq!(DirtyIndicator::symbol(true, true), "🔒"); // Read-only takes precedence
    }

    #[test]
    fn test_status_text() {
        assert_eq!(DirtyIndicator::status_text(true, false), "Modified");
        assert_eq!(DirtyIndicator::status_text(false, false), "Saved");
        assert_eq!(DirtyIndicator::status_text(false, true), "Read Only");
    }

    #[test]
    fn test_short_label() {
        assert_eq!(DirtyIndicator::short_label(true, false), "[+]");
        assert_eq!(DirtyIndicator::short_label(false, false), "");
        assert_eq!(DirtyIndicator::short_label(false, true), "RO");
    }

    #[test]
    fn test_should_highlight() {
        assert!(DirtyIndicator::should_highlight(true));
        assert!(!DirtyIndicator::should_highlight(false));
    }
}
