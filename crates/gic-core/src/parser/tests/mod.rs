//! Comprehensive integration & unit tests for the Universal Language Parsing Engine.

use crate::parser::*;
use std::path::Path;

#[test]
fn test_all_v1_languages_parsing() {
    let engine = ParsingEngine::new();

    // 1. YAML
    let yaml_src = "apiVersion: apps/v1\nkind: Deployment\n# comment\n- item1\n";
    let yaml_tree = engine
        .parse_source(Some(Path::new("deploy.yaml")), yaml_src, None)
        .unwrap();
    assert_eq!(yaml_tree.language, LanguageId::Yaml);
    assert!(!yaml_tree.tokens.is_empty());

    // 2. Dockerfile
    let docker_src = "FROM alpine:3.18\nRUN apk add --no-cache bash\nCMD [\"bash\"]\n";
    let docker_tree = engine
        .parse_source(Some(Path::new("Dockerfile")), docker_src, None)
        .unwrap();
    assert_eq!(docker_tree.language, LanguageId::Dockerfile);

    // 3. Terraform HCL
    let tf_src = "resource \"aws_s3_bucket\" \"b\" {\n  bucket = \"my-tf-test-bucket\"\n}\n";
    let tf_tree = engine
        .parse_source(Some(Path::new("main.tf")), tf_src, None)
        .unwrap();
    assert_eq!(tf_tree.language, LanguageId::Terraform);

    // 4. Bash
    let bash_src = "#!/bin/bash\nif [ -f \"file\" ]; then\n  echo \"found\"\nfi\n";
    let bash_tree = engine
        .parse_source(Some(Path::new("script.sh")), bash_src, None)
        .unwrap();
    assert_eq!(bash_tree.language, LanguageId::Bash);

    // 5. JSON
    let json_src = "{\n  \"name\": \"gic\",\n  \"version\": 1\n}\n";
    let json_tree = engine
        .parse_source(Some(Path::new("package.json")), json_src, None)
        .unwrap();
    assert_eq!(json_tree.language, LanguageId::Json);
    assert!(!json_tree.has_errors());

    // 6. TOML
    let toml_src = "[package]\nname = \"gic\"\nversion = \"0.1.0\"\n";
    let toml_tree = engine
        .parse_source(Some(Path::new("Cargo.toml")), toml_src, None)
        .unwrap();
    assert_eq!(toml_tree.language, LanguageId::Toml);

    // 7. Markdown
    let md_src = "# Heading 1\n\nParagraph text\n- list item 1\n";
    let md_tree = engine
        .parse_source(Some(Path::new("README.md")), md_src, None)
        .unwrap();
    assert_eq!(md_tree.language, LanguageId::Markdown);

    // 8. INI
    let ini_src = "[owner]\nname = John Doe\norganization = Acme Widgets\n";
    let ini_tree = engine
        .parse_source(Some(Path::new("config.ini")), ini_src, None)
        .unwrap();
    assert_eq!(ini_tree.language, LanguageId::Ini);

    // 9. XML
    let xml_src = "<config><setting key=\"test\">value</setting></config>\n";
    let xml_tree = engine
        .parse_source(Some(Path::new("config.xml")), xml_src, None)
        .unwrap();
    assert_eq!(xml_tree.language, LanguageId::Xml);

    // 10. Plain Text
    let txt_src = "Log entry 1\nLog entry 2\n";
    let txt_tree = engine
        .parse_source(Some(Path::new("output.log")), txt_src, None)
        .unwrap();
    assert_eq!(txt_tree.language, LanguageId::PlainText);
}

#[test]
fn test_error_recovery_non_panicking() {
    let engine = ParsingEngine::new();

    // Invalid JSON trailing comma
    let broken_json = "{\n  \"key\": \"value\",\n}\n";
    let tree = engine
        .parse_source(Some(Path::new("test.json")), broken_json, None)
        .unwrap();
    assert!(tree.has_errors());
    assert!(!tree.diagnostics.is_empty());

    // Invalid Dockerfile instruction
    let broken_docker = "UNKNOWN_INSTRUCTION arg1 arg2\n";
    let docker_tree = engine
        .parse_source(Some(Path::new("Dockerfile")), broken_docker, None)
        .unwrap();
    assert!(docker_tree.has_errors());

    // Unclosed XML tag
    let broken_xml = "<open_tag>content\n";
    let xml_tree = engine
        .parse_source(Some(Path::new("test.xml")), broken_xml, None)
        .unwrap();
    assert!(xml_tree.has_errors());
}

#[test]
fn test_tree_traversal_and_symbols() {
    let engine = ParsingEngine::new();
    let toml_src = "[table_one]\nkey1 = \"val1\"\n\n[table_two]\nkey2 = \"val2\"\n";
    let tree = engine
        .parse_source(Some(Path::new("config.toml")), toml_src, None)
        .unwrap();

    let symbols = tree.symbols();
    assert!(symbols.iter().any(|s| s.name.contains("table_one")));
    assert!(symbols.iter().any(|s| s.name.contains("table_two")));
}

#[test]
fn test_incremental_parsing_update() {
    let engine = ParsingEngine::new();
    let v1_src = "key1: val1\n";
    let key = "file1.yaml";

    let initial_tree = engine
        .parse_source(Some(Path::new(key)), v1_src, None)
        .unwrap();

    let v2_src = "key1: val1\nkey2: val2\n";
    let edit_change = TextChange::new(
        TextRange::new(Position::new(1, 0, 11), Position::new(1, 0, 11)),
        "key2: val2\n",
    );

    let updated_tree = engine
        .parse_incremental(key, Some(Path::new(key)), v2_src, &edit_change, None)
        .unwrap();

    assert_eq!(updated_tree.language, LanguageId::Yaml);
    assert_ne!(initial_tree.source_hash, updated_tree.source_hash);
}
