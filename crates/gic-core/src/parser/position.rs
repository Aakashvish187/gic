//! Source text position, range, and text edit representations for the language parsing engine.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Represents a point location within a source text document.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Position {
    /// Zero-based line number.
    pub line: usize,
    /// Zero-based column index (in UTF-8 character units or bytes).
    pub column: usize,
    /// Absolute UTF-8 byte offset in the source document.
    pub byte_offset: usize,
}

impl Position {
    /// Creates a new `Position`.
    pub const fn new(line: usize, column: usize, byte_offset: usize) -> Self {
        Self {
            line,
            column,
            byte_offset,
        }
    }

    /// Creates a zero position at the start of a document.
    pub const fn zero() -> Self {
        Self {
            line: 0,
            column: 0,
            byte_offset: 0,
        }
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}:{}(byte {})",
            self.line + 1,
            self.column + 1,
            self.byte_offset
        )
    }
}

/// Represents a contiguous span of text within a source document.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TextRange {
    /// Start position of the range (inclusive).
    pub start: Position,
    /// End position of the range (exclusive).
    pub end: Position,
}

impl TextRange {
    /// Creates a new `TextRange`.
    pub const fn new(start: Position, end: Position) -> Self {
        Self { start, end }
    }

    /// Creates an empty range at a specific position.
    pub const fn empty(at: Position) -> Self {
        Self { start: at, end: at }
    }

    /// Checks if a given position falls within this range (inclusive start, exclusive end).
    pub fn contains_position(&self, pos: Position) -> bool {
        pos >= self.start && pos < self.end
    }

    /// Checks if two ranges overlap.
    pub fn intersects(&self, other: &TextRange) -> bool {
        self.start < other.end && other.start < self.end
    }

    /// Calculates byte length of this range.
    pub fn byte_len(&self) -> usize {
        self.end.byte_offset.saturating_sub(self.start.byte_offset)
    }

    /// Returns `true` if the range spans 0 bytes.
    pub fn is_empty(&self) -> bool {
        self.byte_len() == 0
    }
}

impl fmt::Display for TextRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}..{}]", self.start, self.end)
    }
}

/// Represents an edit operation on a document range, used for incremental parsing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextChange {
    /// The range of existing text that was replaced or removed.
    pub range: TextRange,
    /// The new text inserted in place of `range`.
    pub new_text: String,
}

impl TextChange {
    /// Creates a new `TextChange`.
    pub fn new(range: TextRange, new_text: impl Into<String>) -> Self {
        Self {
            range,
            new_text: new_text.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_ordering() {
        let p1 = Position::new(0, 5, 5);
        let p2 = Position::new(1, 0, 10);
        assert!(p1 < p2);
        assert_eq!(p1.to_string(), "1:6(byte 5)");
    }

    #[test]
    fn test_range_contains_and_intersects() {
        let r1 = TextRange::new(Position::new(0, 0, 0), Position::new(2, 10, 50));
        let p_inside = Position::new(1, 5, 25);
        let p_outside = Position::new(3, 0, 60);

        assert!(r1.contains_position(p_inside));
        assert!(!r1.contains_position(p_outside));

        let r2 = TextRange::new(Position::new(1, 0, 20), Position::new(4, 0, 80));
        assert!(r1.intersects(&r2));
    }
}
