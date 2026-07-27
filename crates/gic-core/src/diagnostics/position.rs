//! Source position representations tailored for diagnostic location reporting.

use crate::parser::position::Position as ParserPosition;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Represents a 1-based human-readable line/column position with an absolute byte offset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct DiagnosticPosition {
    /// 1-based line number.
    pub line: usize,
    /// 1-based column number.
    pub column: usize,
    /// 0-based absolute UTF-8 byte offset in the source buffer.
    pub byte_offset: usize,
}

impl DiagnosticPosition {
    /// Creates a new `DiagnosticPosition` (accepts 1-based line & column).
    pub fn new(line: usize, column: usize, byte_offset: usize) -> Self {
        Self {
            line: if line == 0 { 1 } else { line },
            column: if column == 0 { 1 } else { column },
            byte_offset,
        }
    }

    /// Creates a position from 0-based parser line and column.
    pub fn from_zero_based(line: usize, column: usize, byte_offset: usize) -> Self {
        Self {
            line: line + 1,
            column: column + 1,
            byte_offset,
        }
    }

    /// Creates a starting position at line 1, column 1, byte offset 0.
    pub fn zero() -> Self {
        Self {
            line: 1,
            column: 1,
            byte_offset: 0,
        }
    }

    /// Converts to zero-based line index.
    pub fn zero_based_line(&self) -> usize {
        self.line.saturating_sub(1)
    }

    /// Converts to zero-based column index.
    pub fn zero_based_column(&self) -> usize {
        self.column.saturating_sub(1)
    }
}

impl From<ParserPosition> for DiagnosticPosition {
    fn from(pos: ParserPosition) -> Self {
        Self::from_zero_based(pos.line, pos.column, pos.byte_offset)
    }
}

impl From<DiagnosticPosition> for ParserPosition {
    fn from(pos: DiagnosticPosition) -> Self {
        ParserPosition::new(
            pos.zero_based_line(),
            pos.zero_based_column(),
            pos.byte_offset,
        )
    }
}

impl fmt::Display for DiagnosticPosition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diagnostic_position_conversions() {
        let p1 = DiagnosticPosition::new(10, 5, 120);
        assert_eq!(p1.zero_based_line(), 9);
        assert_eq!(p1.zero_based_column(), 4);
        assert_eq!(p1.to_string(), "10:5");

        let parser_pos = ParserPosition::new(0, 0, 0);
        let diag_pos: DiagnosticPosition = parser_pos.into();
        assert_eq!(diag_pos.line, 1);
        assert_eq!(diag_pos.column, 1);
    }
}
