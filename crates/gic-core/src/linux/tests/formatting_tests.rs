//! Unit tests for Formatter.

use crate::linux::formatter::LinuxFormatter;

#[test]
fn test_formatter_trims_trailing_whitespace() {
    let source = "echo 'hello'   \nls -la  ";
    let formatter = LinuxFormatter::new();
    let formatted = formatter.format(source);

    assert_eq!(formatted, "echo 'hello'\nls -la\n");
}
