//! Range representations for diagnostics, mapping precise source spans.

use crate::diagnostics::position::DiagnosticPosition;
use crate::parser::position::TextRange as ParserTextRange;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Represents a source code range associated with a diagnostic report.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DiagnosticRange {
    /// Starting position (inclusive).
    pub start: DiagnosticPosition,
    /// Ending position (exclusive).
    pub end: DiagnosticPosition,
}

impl DiagnosticRange {
    /// Creates a new `DiagnosticRange`.
    pub fn new(start: DiagnosticPosition, end: DiagnosticPosition) -> Self {
        Self { start, end }
    }

    /// Creates a single point zero-length range at a given position.
    pub fn point(at: DiagnosticPosition) -> Self {
        Self { start: at, end: at }
    }

    /// Creates a range restricted to a single line.
    pub fn single_line(
        line: usize,
        start_col: usize,
        end_col: usize,
        start_byte: usize,
        end_byte: usize,
    ) -> Self {
        Self {
            start: DiagnosticPosition::new(line, start_col, start_byte),
            end: DiagnosticPosition::new(line, end_col, end_byte),
        }
    }

    /// Checks if a position falls within this range.
    pub fn contains_position(&self, pos: DiagnosticPosition) -> bool {
        pos >= self.start && pos < self.end
    }

    /// Checks if two ranges overlap.
    pub fn intersects(&self, other: &DiagnosticRange) -> bool {
        self.start < other.end && other.start < self.end
    }

    /// Returns byte length spanned by this range.
    pub fn byte_len(&self) -> usize {
        self.end.byte_offset.saturating_sub(self.start.byte_offset)
    }

    /// Returns `true` if the range spans 0 bytes.
    pub fn is_empty(&self) -> bool {
        self.byte_len() == 0
    }
}

impl From<ParserTextRange> for DiagnosticRange {
    fn from(r: ParserTextRange) -> Self {
        Self {
            start: r.start.into(),
            end: r.end.into(),
        }
    }
}

impl From<DiagnosticRange> for ParserTextRange {
    fn from(r: DiagnosticRange) -> Self {
        ParserTextRange::new(r.start.into(), r.end.into())
    }
}

impl fmt::Display for DiagnosticRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}..{}]", self.start, self.end)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diagnostic_range_operations() {
        let p1 = DiagnosticPosition::new(1, 1, 0);
        let p2 = DiagnosticPosition::new(1, 10, 9);
        let r1 = DiagnosticRange::new(p1, p2);

        assert_eq!(r1.byte_len(), 9);
        assert!(!r1.is_empty());
        assert!(r1.contains_position(DiagnosticPosition::new(1, 5, 4)));
        assert!(!r1.contains_position(DiagnosticPosition::new(2, 1, 10)));
    }
}
