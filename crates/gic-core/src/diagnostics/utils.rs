//! Helper utilities for diagnostic range calculation, timestamp generation, and AST inspection.

use crate::diagnostics::position::DiagnosticPosition;
use crate::diagnostics::range::DiagnosticRange;
use std::time::{SystemTime, UNIX_EPOCH};

/// Generates a deterministic or unique diagnostic ID based on rule, location, and message hash.
pub fn generate_diagnostic_id(
    rule_name: &str,
    line: usize,
    column: usize,
    message: &str,
) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    rule_name.hash(&mut hasher);
    line.hash(&mut hasher);
    column.hash(&mut hasher);
    message.hash(&mut hasher);
    let hash = hasher.finish();

    format!(
        "{}-{:08x}",
        rule_name.to_lowercase().replace(' ', "-"),
        hash & 0xffff_ffff
    )
}

/// Returns current system timestamp in milliseconds since UNIX_EPOCH.
pub fn current_timestamp_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// Computes line and column (1-based) for a given byte offset within source text.
pub fn offset_to_position(source: &str, offset: usize) -> DiagnosticPosition {
    let offset = offset.min(source.len());
    let mut line = 1;
    let mut last_line_start = 0;

    for (idx, b) in source.bytes().take(offset).enumerate() {
        if b == b'\n' {
            line += 1;
            last_line_start = idx + 1;
        }
    }

    let column = source[last_line_start..offset].chars().count() + 1;
    DiagnosticPosition::new(line, column, offset)
}

/// Computes a `DiagnosticRange` spanning from `start_offset` to `end_offset` in source text.
pub fn offsets_to_range(source: &str, start_offset: usize, end_offset: usize) -> DiagnosticRange {
    let start_pos = offset_to_position(source, start_offset);
    let end_pos = offset_to_position(source, end_offset.max(start_offset));
    DiagnosticRange::new(start_pos, end_pos)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_diagnostic_id() {
        let id1 = generate_diagnostic_id("NoTabIndent", 10, 2, "Tab found");
        let id2 = generate_diagnostic_id("NoTabIndent", 10, 2, "Tab found");
        assert_eq!(id1, id2);
        assert!(id1.starts_with("notabindent-"));
    }

    #[test]
    fn test_offset_to_position_multiline() {
        let text = "hello\nworld\nfoo";
        let pos1 = offset_to_position(text, 0);
        assert_eq!(pos1.line, 1);
        assert_eq!(pos1.column, 1);

        let pos2 = offset_to_position(text, 6); // start of "world"
        assert_eq!(pos2.line, 2);
        assert_eq!(pos2.column, 1);
    }
}
