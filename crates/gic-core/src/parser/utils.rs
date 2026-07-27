//! General helper utilities for string hash generation and source position indexing.

use crate::parser::position::Position;
use std::hash::{Hash, Hasher};

/// Computes a fast 64-bit FNV-1a hash of a source string for cache keying.
pub fn hash_source(source: &str) -> u64 {
    let mut hasher = fnv::FnvHasher::default();
    source.hash(&mut hasher);
    hasher.finish()
}

/// Computes zero-based byte start offsets for each line in the source text.
pub fn compute_line_offsets(source: &str) -> Vec<usize> {
    let mut offsets = vec![0];
    for (idx, b) in source.bytes().enumerate() {
        if b == b'\n' {
            offsets.push(idx + 1);
        }
    }
    offsets
}

/// Converts a byte offset in `source` into a `Position` (line, column, byte_offset).
pub fn byte_offset_to_position(line_offsets: &[usize], offset: usize) -> Position {
    if line_offsets.is_empty() {
        return Position::zero();
    }

    let line = match line_offsets.binary_search(&offset) {
        Ok(exact) => exact,
        Err(0) => 0,
        Err(idx) => idx - 1,
    };

    let line_start = line_offsets[line];
    let column = offset.saturating_sub(line_start);

    Position::new(line, column, offset)
}

mod fnv {
    use std::hash::Hasher;

    pub struct FnvHasher(u64);

    impl Default for FnvHasher {
        fn default() -> Self {
            FnvHasher(0xcbf29ce484222325)
        }
    }

    impl Hasher for FnvHasher {
        fn finish(&self) -> u64 {
            self.0
        }

        fn write(&mut self, bytes: &[u8]) {
            for &byte in bytes {
                self.0 ^= u64::from(byte);
                self.0 = self.0.wrapping_mul(0x100000001b3);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_line_offsets_and_position() {
        let source = "line 1\nline 2\nline 3";
        let offsets = compute_line_offsets(source);
        assert_eq!(offsets, vec![0, 7, 14]);

        let pos1 = byte_offset_to_position(&offsets, 2);
        assert_eq!(pos1.line, 0);
        assert_eq!(pos1.column, 2);

        let pos2 = byte_offset_to_position(&offsets, 9);
        assert_eq!(pos2.line, 1);
        assert_eq!(pos2.column, 2);
    }
}
