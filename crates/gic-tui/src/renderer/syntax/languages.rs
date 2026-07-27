//! # Language Definitions
//!
//! Defines language-specific syntax rules (keywords, patterns) and provides
//! a registry for resolving file extensions to language definitions.
//!
//! Adding a new language requires only adding a new `LanguageDefinition`
//! constant and registering it in the `LanguageRegistry` — no existing
//! code needs to change.

use std::collections::HashMap;

/// Defines the syntax rules for a single programming language.
///
/// Each field contains patterns or keywords that the regex highlighter
/// uses to tokenize source code.
#[derive(Debug, Clone)]
pub struct LanguageDefinition {
    /// Human-readable language name.
    pub name: &'static str,
    /// File extensions this language covers (without leading dot).
    pub extensions: &'static [&'static str],
    /// Language keywords (e.g., `fn`, `let`, `if`).
    pub keywords: &'static [&'static str],
    /// Built-in type names (e.g., `String`, `i32`, `bool`).
    pub types: &'static [&'static str],
    /// Built-in constant values (e.g., `true`, `false`, `None`).
    pub constants: &'static [&'static str],
    /// Single-line comment prefix (e.g., `//`, `#`).
    pub line_comment: &'static str,
    /// Block comment start delimiter (e.g., `/*`). Empty if not supported.
    pub block_comment_start: &'static str,
    /// Block comment end delimiter (e.g., `*/`). Empty if not supported.
    pub block_comment_end: &'static str,
    /// String delimiter characters (e.g., `"`, `'`).
    pub string_delimiters: &'static [char],
    /// Whether the language supports raw strings (e.g., `r"..."` in Rust).
    pub has_raw_strings: bool,
}

// ─── Built-in Language Definitions ──────────────────────────────────

/// Rust language definition.
pub static RUST: LanguageDefinition = LanguageDefinition {
    name: "Rust",
    extensions: &["rs"],
    keywords: &[
        "as", "async", "await", "break", "const", "continue", "crate", "dyn", "else", "enum",
        "extern", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut",
        "pub", "ref", "return", "self", "Self", "static", "struct", "super", "trait", "type",
        "union", "unsafe", "use", "where", "while", "yield",
    ],
    types: &[
        "bool", "char", "f32", "f64", "i8", "i16", "i32", "i64", "i128", "isize", "str", "u8",
        "u16", "u32", "u64", "u128", "usize", "String", "Vec", "Option", "Result", "Box", "Rc",
        "Arc", "HashMap", "HashSet", "BTreeMap", "BTreeSet",
    ],
    constants: &["true", "false", "None", "Some", "Ok", "Err"],
    line_comment: "//",
    block_comment_start: "/*",
    block_comment_end: "*/",
    string_delimiters: &['"'],
    has_raw_strings: true,
};

/// YAML language definition.
pub static YAML: LanguageDefinition = LanguageDefinition {
    name: "YAML",
    extensions: &["yml", "yaml"],
    keywords: &[],
    types: &[],
    constants: &["true", "false", "null", "yes", "no", "on", "off"],
    line_comment: "#",
    block_comment_start: "",
    block_comment_end: "",
    string_delimiters: &['"', '\''],
    has_raw_strings: false,
};

/// JSON language definition.
pub static JSON: LanguageDefinition = LanguageDefinition {
    name: "JSON",
    extensions: &["json", "jsonc", "geojson"],
    keywords: &[],
    types: &[],
    constants: &["true", "false", "null"],
    line_comment: "",
    block_comment_start: "",
    block_comment_end: "",
    string_delimiters: &['"'],
    has_raw_strings: false,
};

/// TOML language definition.
pub static TOML: LanguageDefinition = LanguageDefinition {
    name: "TOML",
    extensions: &["toml"],
    keywords: &[],
    types: &[],
    constants: &["true", "false"],
    line_comment: "#",
    block_comment_start: "",
    block_comment_end: "",
    string_delimiters: &['"', '\''],
    has_raw_strings: false,
};

/// Dockerfile language definition.
pub static DOCKERFILE: LanguageDefinition = LanguageDefinition {
    name: "Dockerfile",
    extensions: &["dockerfile"],
    keywords: &[
        "FROM",
        "RUN",
        "CMD",
        "LABEL",
        "MAINTAINER",
        "EXPOSE",
        "ENV",
        "ADD",
        "COPY",
        "ENTRYPOINT",
        "VOLUME",
        "USER",
        "WORKDIR",
        "ARG",
        "ONBUILD",
        "STOPSIGNAL",
        "HEALTHCHECK",
        "SHELL",
    ],
    types: &[],
    constants: &[],
    line_comment: "#",
    block_comment_start: "",
    block_comment_end: "",
    string_delimiters: &['"', '\''],
    has_raw_strings: false,
};

/// Terraform (HCL) language definition.
pub static TERRAFORM: LanguageDefinition = LanguageDefinition {
    name: "Terraform",
    extensions: &["tf", "tfvars", "hcl"],
    keywords: &[
        "resource",
        "data",
        "variable",
        "output",
        "module",
        "provider",
        "terraform",
        "locals",
        "dynamic",
        "for_each",
        "count",
        "depends_on",
        "lifecycle",
        "provisioner",
        "connection",
    ],
    types: &[
        "string", "number", "bool", "list", "map", "set", "object", "tuple", "any",
    ],
    constants: &["true", "false", "null"],
    line_comment: "#",
    block_comment_start: "/*",
    block_comment_end: "*/",
    string_delimiters: &['"'],
    has_raw_strings: false,
};

/// Shell / Bash language definition.
pub static SHELL: LanguageDefinition = LanguageDefinition {
    name: "Shell",
    extensions: &["sh", "bash", "zsh", "fish", "ksh"],
    keywords: &[
        "if", "then", "else", "elif", "fi", "for", "while", "do", "done", "case", "esac", "in",
        "function", "return", "exit", "local", "export", "source", "alias", "unalias", "set",
        "unset", "readonly", "declare", "typeset", "shift", "trap",
    ],
    types: &[],
    constants: &["true", "false"],
    line_comment: "#",
    block_comment_start: "",
    block_comment_end: "",
    string_delimiters: &['"', '\''],
    has_raw_strings: false,
};

/// Markdown language definition.
pub static MARKDOWN: LanguageDefinition = LanguageDefinition {
    name: "Markdown",
    extensions: &["md", "markdown", "mdown", "mkd"],
    keywords: &[],
    types: &[],
    constants: &[],
    line_comment: "",
    block_comment_start: "",
    block_comment_end: "",
    string_delimiters: &[],
    has_raw_strings: false,
};

/// Plain text — fallback definition.
pub static PLAIN_TEXT: LanguageDefinition = LanguageDefinition {
    name: "Plain Text",
    extensions: &["txt", "text", "log"],
    keywords: &[],
    types: &[],
    constants: &[],
    line_comment: "",
    block_comment_start: "",
    block_comment_end: "",
    string_delimiters: &[],
    has_raw_strings: false,
};

// ─── Language Registry ──────────────────────────────────────────────

/// Registry that maps file extensions to language definitions.
///
/// The registry is initialized with all built-in languages and can
/// resolve a file extension to the appropriate `LanguageDefinition`.
pub struct LanguageRegistry {
    /// Extension → language definition mapping.
    extensions: HashMap<String, &'static LanguageDefinition>,
    /// Filename → language definition mapping (for files like "Dockerfile").
    filenames: HashMap<String, &'static LanguageDefinition>,
}

impl Default for LanguageRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl LanguageRegistry {
    /// Creates a new registry pre-populated with all built-in languages.
    pub fn new() -> Self {
        let mut extensions = HashMap::new();
        let mut filenames = HashMap::new();

        let all_languages: &[&LanguageDefinition] = &[
            &RUST,
            &YAML,
            &JSON,
            &TOML,
            &DOCKERFILE,
            &TERRAFORM,
            &SHELL,
            &MARKDOWN,
            &PLAIN_TEXT,
        ];

        for lang in all_languages {
            for ext in lang.extensions {
                extensions.insert(ext.to_lowercase(), *lang);
            }
        }

        // Special filename mappings
        filenames.insert("dockerfile".to_string(), &DOCKERFILE);
        filenames.insert("makefile".to_string(), &SHELL);
        filenames.insert("justfile".to_string(), &SHELL);

        Self {
            extensions,
            filenames,
        }
    }

    /// Resolves a file extension to a language definition.
    ///
    /// Returns `None` if no language matches the extension.
    pub fn resolve_by_extension(&self, extension: &str) -> Option<&'static LanguageDefinition> {
        self.extensions.get(&extension.to_lowercase()).copied()
    }

    /// Resolves a filename to a language definition.
    ///
    /// Checks filename mappings first, then falls back to extension matching.
    pub fn resolve_by_filename(&self, filename: &str) -> Option<&'static LanguageDefinition> {
        // Check direct filename match first
        let lower = filename.to_lowercase();
        if let Some(lang) = self.filenames.get(&lower) {
            return Some(lang);
        }

        // Fall back to extension
        if let Some(dot_pos) = filename.rfind('.') {
            let ext = &filename[dot_pos + 1..];
            self.resolve_by_extension(ext)
        } else {
            None
        }
    }

    /// Returns all registered language names.
    pub fn available_languages(&self) -> Vec<&'static str> {
        let mut names: Vec<&'static str> = self.extensions.values().map(|lang| lang.name).collect();
        names.sort_unstable();
        names.dedup();
        names
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rust_definition() {
        assert_eq!(RUST.name, "Rust");
        assert!(RUST.keywords.contains(&"fn"));
        assert!(RUST.types.contains(&"String"));
        assert!(RUST.constants.contains(&"true"));
        assert_eq!(RUST.line_comment, "//");
    }

    #[test]
    fn test_yaml_definition() {
        assert_eq!(YAML.name, "YAML");
        assert!(YAML.constants.contains(&"null"));
        assert_eq!(YAML.line_comment, "#");
    }

    #[test]
    fn test_registry_resolve_by_extension() {
        let registry = LanguageRegistry::new();

        let rust = registry.resolve_by_extension("rs");
        assert!(rust.is_some());
        assert_eq!(rust.unwrap().name, "Rust");

        let yaml = registry.resolve_by_extension("yml");
        assert!(yaml.is_some());
        assert_eq!(yaml.unwrap().name, "YAML");

        let yaml2 = registry.resolve_by_extension("yaml");
        assert!(yaml2.is_some());
        assert_eq!(yaml2.unwrap().name, "YAML");

        let unknown = registry.resolve_by_extension("xyz");
        assert!(unknown.is_none());
    }

    #[test]
    fn test_registry_resolve_by_filename() {
        let registry = LanguageRegistry::new();

        let docker = registry.resolve_by_filename("Dockerfile");
        assert!(docker.is_some());
        assert_eq!(docker.unwrap().name, "Dockerfile");

        let rust = registry.resolve_by_filename("main.rs");
        assert!(rust.is_some());
        assert_eq!(rust.unwrap().name, "Rust");

        let toml = registry.resolve_by_filename("Cargo.toml");
        assert!(toml.is_some());
        assert_eq!(toml.unwrap().name, "TOML");
    }

    #[test]
    fn test_registry_case_insensitive() {
        let registry = LanguageRegistry::new();

        let rs = registry.resolve_by_extension("RS");
        assert!(rs.is_some());
        assert_eq!(rs.unwrap().name, "Rust");
    }

    #[test]
    fn test_available_languages() {
        let registry = LanguageRegistry::new();
        let langs = registry.available_languages();

        assert!(langs.contains(&"Rust"));
        assert!(langs.contains(&"YAML"));
        assert!(langs.contains(&"JSON"));
        assert!(langs.contains(&"Shell"));
        assert!(langs.contains(&"Terraform"));
    }

    #[test]
    fn test_all_language_definitions_valid() {
        let all = &[
            &RUST,
            &YAML,
            &JSON,
            &TOML,
            &DOCKERFILE,
            &TERRAFORM,
            &SHELL,
            &MARKDOWN,
            &PLAIN_TEXT,
        ];
        for lang in all {
            assert!(!lang.name.is_empty());
            assert!(!lang.extensions.is_empty());
        }
    }
}
