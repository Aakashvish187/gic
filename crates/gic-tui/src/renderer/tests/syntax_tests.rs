//! Syntax highlighting tests.

use crate::renderer::syntax::highlighter::{PlainTextHighlighter, SyntaxHighlighter};
use crate::renderer::syntax::languages::*;
use crate::renderer::syntax::regex_highlighter::RegexHighlighter;
use crate::renderer::syntax::token::TokenKind;

#[test]
fn test_rust_full_function() {
    let h = RegexHighlighter::new(&RUST);
    let tokens = h.highlight_line("pub fn add(a: i32, b: i32) -> i32 {", 0);

    assert!(tokens
        .iter()
        .any(|t| t.kind == TokenKind::Keyword && t.text == "pub"));
    assert!(tokens
        .iter()
        .any(|t| t.kind == TokenKind::Keyword && t.text == "fn"));
    assert!(tokens
        .iter()
        .any(|t| t.kind == TokenKind::Function && t.text == "add"));
    assert!(tokens
        .iter()
        .any(|t| t.kind == TokenKind::Type && t.text == "i32"));
}

#[test]
fn test_rust_use_statement() {
    let h = RegexHighlighter::new(&RUST);
    let tokens = h.highlight_line("use std::collections::HashMap;", 0);

    assert!(tokens
        .iter()
        .any(|t| t.kind == TokenKind::Keyword && t.text == "use"));
    assert!(tokens
        .iter()
        .any(|t| t.kind == TokenKind::Type && t.text == "HashMap"));
}

#[test]
fn test_yaml_key_value() {
    let h = RegexHighlighter::new(&YAML);
    let tokens = h.highlight_line("name: \"gic\"", 0);

    assert!(tokens
        .iter()
        .any(|t| t.kind == TokenKind::String && t.text == "\"gic\""));
}

#[test]
fn test_dockerfile_multi_keyword() {
    let h = RegexHighlighter::new(&DOCKERFILE);
    let tokens = h.highlight_line("COPY --from=builder /app /app", 0);

    assert!(tokens
        .iter()
        .any(|t| t.kind == TokenKind::Keyword && t.text == "COPY"));
}

#[test]
fn test_terraform_resource() {
    let h = RegexHighlighter::new(&TERRAFORM);
    let tokens = h.highlight_line("resource \"aws_instance\" \"web\" {", 0);

    assert!(tokens
        .iter()
        .any(|t| t.kind == TokenKind::Keyword && t.text == "resource"));
    assert!(tokens.iter().any(|t| t.kind == TokenKind::String));
}

#[test]
fn test_shell_function() {
    let h = RegexHighlighter::new(&SHELL);
    let tokens = h.highlight_line("function deploy() {", 0);

    assert!(tokens
        .iter()
        .any(|t| t.kind == TokenKind::Keyword && t.text == "function"));
}

#[test]
fn test_language_registry_all_extensions() {
    let registry = LanguageRegistry::new();

    let test_cases = vec![
        ("rs", "Rust"),
        ("yml", "YAML"),
        ("yaml", "YAML"),
        ("json", "JSON"),
        ("toml", "TOML"),
        ("tf", "Terraform"),
        ("sh", "Shell"),
        ("bash", "Shell"),
        ("md", "Markdown"),
        ("txt", "Plain Text"),
    ];

    for (ext, expected_lang) in test_cases {
        let resolved = registry.resolve_by_extension(ext);
        assert!(resolved.is_some(), "Extension '{}' should resolve", ext);
        assert_eq!(
            resolved.unwrap().name,
            expected_lang,
            "Extension '{}' resolved to wrong language",
            ext
        );
    }
}

#[test]
fn test_plain_text_highlighter_preserves_content() {
    let h = PlainTextHighlighter;
    let input = "  Hello   World  ";
    let tokens = h.highlight_line(input, 0);

    let reconstructed: String = tokens.iter().map(|t| t.text.as_str()).collect();
    assert_eq!(reconstructed, input);
}

#[test]
fn test_regex_highlighter_unicode_line() {
    let h = RegexHighlighter::new(&RUST);
    let tokens = h.highlight_line("let emoji = \"🦀\";", 0);

    let reconstructed: String = tokens.iter().map(|t| t.text.as_str()).collect();
    assert_eq!(reconstructed, "let emoji = \"🦀\";");
}
