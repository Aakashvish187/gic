//! Unit tests for YAML Formatter Engine.

use crate::yaml::formatter::{LineEnding, YamlFormatter, YamlFormatterOptions};

#[test]
fn test_formatter_tab_conversion() {
    let source = "server:\n\tport: 8080";
    let formatter = YamlFormatter::default();
    let formatted = formatter.format(source).unwrap();

    assert!(!formatted.contains('\t'));
    assert!(formatted.contains("  port: 8080"));
}

#[test]
fn test_formatter_trailing_whitespace_and_blank_lines() {
    let source = "key1: val1   \n\n\nkey2: val2  ";
    let options = YamlFormatterOptions {
        indent_step: 2,
        trim_trailing_whitespace: true,
        normalize_blank_lines: true,
        preserve_comments: true,
        line_ending: LineEnding::Lf,
    };
    let formatter = YamlFormatter::new(options);
    let formatted = formatter.format(source).unwrap();

    assert!(!formatted.contains("val1   "));
    assert!(!formatted.contains("\n\n\n"));
}

#[test]
fn test_formatter_colon_spacing_fix() {
    let source = "key:value";
    let formatter = YamlFormatter::default();
    let formatted = formatter.format(source).unwrap();
    assert!(formatted.contains("key: value"));
}
