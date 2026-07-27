//! Language identification, specification metadata, and automatic detection logic.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::Path;

/// Supported language identifiers for the GIC parsing engine.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum LanguageId {
    // --- V1 Supported Languages ---
    /// YAML format (.yaml, .yml)
    Yaml,
    /// Dockerfile format (Dockerfile, Dockerfile.*, Containerfile)
    Dockerfile,
    /// Terraform / HCL (.tf, .tfvars, .hcl)
    Terraform,
    /// Bash / Shell script (.sh, .bash, shebang #!/bin/bash)
    Bash,
    /// JSON format (.json)
    Json,
    /// TOML format (.toml)
    Toml,
    /// Markdown format (.md, .markdown)
    Markdown,
    /// INI configuration (.ini, .cfg, .conf)
    Ini,
    /// XML format (.xml)
    Xml,
    /// Plain Text (.txt, default fallback)
    PlainText,

    // --- Future Ready Language Architecture ---
    Python,
    Go,
    Rust,
    Java,
    C,
    Cpp,
    JavaScript,
    TypeScript,
    Html,
    Css,
    Sql,
    Ansible,
    Helm,
    Compose,
    Kustomize,
    Nginx,
    Apache,
    Systemd,

    /// User-defined or custom language plugin
    Custom(String),
}

impl LanguageId {
    /// Returns the canonical display name of the language.
    pub fn display_name(&self) -> &'static str {
        match self {
            LanguageId::Yaml => "YAML",
            LanguageId::Dockerfile => "Dockerfile",
            LanguageId::Terraform => "Terraform (HCL)",
            LanguageId::Bash => "Bash",
            LanguageId::Json => "JSON",
            LanguageId::Toml => "TOML",
            LanguageId::Markdown => "Markdown",
            LanguageId::Ini => "INI",
            LanguageId::Xml => "XML",
            LanguageId::PlainText => "Plain Text",
            LanguageId::Python => "Python",
            LanguageId::Go => "Go",
            LanguageId::Rust => "Rust",
            LanguageId::Java => "Java",
            LanguageId::C => "C",
            LanguageId::Cpp => "C++",
            LanguageId::JavaScript => "JavaScript",
            LanguageId::TypeScript => "TypeScript",
            LanguageId::Html => "HTML",
            LanguageId::Css => "CSS",
            LanguageId::Sql => "SQL",
            LanguageId::Ansible => "Ansible",
            LanguageId::Helm => "Helm",
            LanguageId::Compose => "Docker Compose",
            LanguageId::Kustomize => "Kustomize",
            LanguageId::Nginx => "Nginx Config",
            LanguageId::Apache => "Apache Config",
            LanguageId::Systemd => "Systemd Unit",
            LanguageId::Custom(_) => "Custom Language",
        }
    }

    /// Returns a list of default file extensions associated with this language.
    pub fn extensions(&self) -> &'static [&'static str] {
        match self {
            LanguageId::Yaml => &["yaml", "yml"],
            LanguageId::Dockerfile => &["dockerfile"],
            LanguageId::Terraform => &["tf", "tfvars", "hcl"],
            LanguageId::Bash => &["sh", "bash", "zsh"],
            LanguageId::Json => &["json"],
            LanguageId::Toml => &["toml"],
            LanguageId::Markdown => &["md", "markdown"],
            LanguageId::Ini => &["ini", "cfg", "conf"],
            LanguageId::Xml => &["xml", "xsd", "svg"],
            LanguageId::PlainText => &["txt", "log"],
            LanguageId::Python => &["py", "pyw"],
            LanguageId::Go => &["go"],
            LanguageId::Rust => &["rs"],
            LanguageId::Java => &["java"],
            LanguageId::C => &["c", "h"],
            LanguageId::Cpp => &["cpp", "hpp", "cc", "cxx"],
            LanguageId::JavaScript => &["js", "mjs", "cjs"],
            LanguageId::TypeScript => &["ts", "tsx"],
            LanguageId::Html => &["html", "htm"],
            LanguageId::Css => &["css"],
            LanguageId::Sql => &["sql"],
            LanguageId::Ansible => &["ansible.yml", "ansible.yaml"],
            LanguageId::Helm => &["helm.yaml"],
            LanguageId::Compose => &["docker-compose.yml", "compose.yaml"],
            LanguageId::Kustomize => &["kustomization.yaml"],
            LanguageId::Nginx => &["nginx.conf"],
            LanguageId::Apache => &["httpd.conf"],
            LanguageId::Systemd => &["service", "socket", "target", "timer"],
            LanguageId::Custom(_) => &[],
        }
    }
}

impl fmt::Display for LanguageId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

/// Metadata specification for a language.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LanguageSpec {
    pub id: LanguageId,
    pub name: String,
    pub extensions: Vec<String>,
    pub filenames: Vec<String>,
    pub shebang_patterns: Vec<String>,
}

/// High-performance language auto-detection utility.
#[derive(Debug, Default, Clone)]
pub struct LanguageDetector;

impl LanguageDetector {
    /// Creates a new `LanguageDetector`.
    pub fn new() -> Self {
        Self
    }

    /// Automatically detects language from file path, first line (shebang), or explicit override.
    pub fn detect(
        &self,
        path: Option<&Path>,
        first_line_or_shebang: Option<&str>,
        override_lang: Option<LanguageId>,
    ) -> LanguageId {
        // 1. Manual override takes highest precedence
        if let Some(lang) = override_lang {
            return lang;
        }

        // 2. Exact filename matching
        if let Some(path) = path {
            if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                let lower_name = file_name.to_lowercase();

                if lower_name == "dockerfile"
                    || lower_name.starts_with("dockerfile.")
                    || lower_name == "containerfile"
                {
                    return LanguageId::Dockerfile;
                }
                if lower_name == "docker-compose.yml"
                    || lower_name == "docker-compose.yaml"
                    || lower_name == "compose.yaml"
                {
                    return LanguageId::Yaml;
                }
                if lower_name == "kustomization.yaml" || lower_name == "kustomization.yml" {
                    return LanguageId::Yaml;
                }
            }
        }

        // 3. Shebang detection (if available)
        if let Some(first_line) = first_line_or_shebang {
            let line = first_line.trim();
            if line.starts_with("#!") {
                if line.contains("bash") || line.contains("/sh") || line.contains("zsh") {
                    return LanguageId::Bash;
                }
                if line.contains("python") {
                    return LanguageId::Python;
                }
                if line.contains("node") {
                    return LanguageId::JavaScript;
                }
            }
        }

        // 4. File extension matching
        if let Some(path) = path {
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                let ext_lower = ext.to_lowercase();
                match ext_lower.as_str() {
                    "yaml" | "yml" => return LanguageId::Yaml,
                    "tf" | "tfvars" | "hcl" => return LanguageId::Terraform,
                    "sh" | "bash" | "zsh" => return LanguageId::Bash,
                    "json" => return LanguageId::Json,
                    "toml" => return LanguageId::Toml,
                    "md" | "markdown" => return LanguageId::Markdown,
                    "ini" | "cfg" | "conf" => return LanguageId::Ini,
                    "xml" | "xsd" | "svg" => return LanguageId::Xml,
                    "py" => return LanguageId::Python,
                    "go" => return LanguageId::Go,
                    "rs" => return LanguageId::Rust,
                    "java" => return LanguageId::Java,
                    "c" | "h" => return LanguageId::C,
                    "cpp" | "hpp" | "cc" | "cxx" => return LanguageId::Cpp,
                    "js" | "mjs" | "cjs" => return LanguageId::JavaScript,
                    "ts" | "tsx" => return LanguageId::TypeScript,
                    "html" | "htm" => return LanguageId::Html,
                    "css" => return LanguageId::Css,
                    "sql" => return LanguageId::Sql,
                    _ => {}
                }
            }
        }

        // 5. Default fallback to Plain Text
        LanguageId::PlainText
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_detection() {
        let detector = LanguageDetector::new();

        // Path extension test
        assert_eq!(
            detector.detect(Some(Path::new("deploy.yaml")), None, None),
            LanguageId::Yaml
        );
        assert_eq!(
            detector.detect(Some(Path::new("main.tf")), None, None),
            LanguageId::Terraform
        );

        // Exact filename test
        assert_eq!(
            detector.detect(Some(Path::new("Dockerfile")), None, None),
            LanguageId::Dockerfile
        );
        assert_eq!(
            detector.detect(Some(Path::new("Dockerfile.dev")), None, None),
            LanguageId::Dockerfile
        );

        // Shebang test
        assert_eq!(
            detector.detect(None, Some("#!/bin/bash -e"), None),
            LanguageId::Bash
        );

        // Override test
        assert_eq!(
            detector.detect(Some(Path::new("script.sh")), None, Some(LanguageId::Python)),
            LanguageId::Python
        );
    }
}
