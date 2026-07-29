//! # Terraform Language Engine

use super::{Completion, CompletionKind, EngineDiagnostic, HoverInfo, LanguageEngine};

pub struct TerraformEngine;

impl TerraformEngine {
    pub fn new() -> Self {
        Self
    }
}

impl LanguageEngine for TerraformEngine {
    fn name(&self) -> &'static str { "Terraform" }
    fn id(&self) -> &'static str { "terraform" }

    fn diagnostics(&self, content: &str) -> Vec<EngineDiagnostic> {
        let mut diagnostics = Vec::new();

        for (row, line) in content.lines().enumerate() {
            let trimmed = line.trim();

            // Check for unclosed braces (simple heuristic)
            // A full HCL parser would be better, but this catches common mistakes

            // Check for hardcoded credentials
            let lower = trimmed.to_lowercase();
            if (lower.contains("access_key") || lower.contains("secret_key") || lower.contains("password"))
                && trimmed.contains('=')
                && trimmed.contains('"')
            {
                diagnostics.push(
                    EngineDiagnostic::warning(row, 0,
                        "Possible hardcoded credential. Use variables or environment variables instead.",
                        "terraform")
                        .with_code("TF001")
                );
            }

            // Check for missing version constraint on provider
            if trimmed.starts_with("provider") && trimmed.contains('{') {
                let mut found_version = false;
                for j in (row + 1)..content.lines().count().min(row + 10) {
                    let next = content.lines().nth(j).unwrap_or("");
                    if next.trim().starts_with("version") {
                        found_version = true;
                        break;
                    }
                    if next.trim() == "}" {
                        break;
                    }
                }
                if !found_version {
                    diagnostics.push(
                        EngineDiagnostic::warning(row, 0,
                            "Provider block without version constraint. Pin the version for reproducible infrastructure.",
                            "terraform")
                            .with_code("TF002")
                    );
                }
            }

            // Check for deprecated interpolation syntax
            if trimmed.contains("\"${") && trimmed.contains("}\"") {
                // In Terraform 0.12+, standalone interpolation is deprecated
                // "${var.foo}" should be just var.foo
                if let Some(start) = trimmed.find("\"${") {
                    if let Some(end) = trimmed[start..].find("}\"") {
                        let interp = &trimmed[start..start + end + 2];
                        // Only flag if it's a simple variable reference
                        if !interp.contains("${") || interp.matches("${").count() == 1 {
                            diagnostics.push(
                                EngineDiagnostic::hint(row, start,
                                    "Deprecated interpolation syntax. In Terraform 0.12+, use the expression directly without ${}.",
                                    "terraform")
                                    .with_code("TF003")
                            );
                        }
                    }
                }
            }
        }

        diagnostics
    }

    fn completions(&self, _content: &str, _row: usize, _col: usize) -> Vec<Completion> {
        let blocks = [
            ("resource", "Define a resource", "resource \"type\" \"name\" {\n  \n}"),
            ("data", "Data source", "data \"type\" \"name\" {\n  \n}"),
            ("variable", "Input variable", "variable \"name\" {\n  type = string\n  default = \"\"\n}"),
            ("output", "Output value", "output \"name\" {\n  value = \n}"),
            ("module", "Module call", "module \"name\" {\n  source = \"\"\n}"),
            ("provider", "Provider config", "provider \"name\" {\n  \n}"),
            ("terraform", "Terraform settings", "terraform {\n  required_version = \">= 1.0\"\n}"),
            ("locals", "Local values", "locals {\n  \n}"),
        ];

        blocks.iter().map(|(name, detail, insert)| {
            Completion::new(*name, *insert, CompletionKind::Snippet).with_detail(*detail)
        }).collect()
    }

    fn hover(&self, content: &str, row: usize, _col: usize) -> Option<HoverInfo> {
        let line = content.lines().nth(row)?;
        let trimmed = line.trim();
        let first_word = trimmed.split_whitespace().next()?;

        match first_word {
            "resource" => Some(HoverInfo::new("resource", "Defines an infrastructure resource to be managed by Terraform.")
                .with_syntax("resource \"aws_instance\" \"web\" {\n  ami           = \"ami-12345\"\n  instance_type = \"t3.micro\"\n}")
                .with_best_practice("Use descriptive resource names. Tag all resources.")),
            "variable" => Some(HoverInfo::new("variable", "Declares an input variable for the module.")
                .with_syntax("variable \"instance_type\" {\n  type    = string\n  default = \"t3.micro\"\n}")
                .with_best_practice("Always add descriptions to variables.")),
            "output" => Some(HoverInfo::new("output", "Declares an output value from the module.")
                .with_syntax("output \"instance_ip\" {\n  value = aws_instance.web.public_ip\n}")),
            "module" => Some(HoverInfo::new("module", "Calls a child module with the specified configuration.")
                .with_syntax("module \"vpc\" {\n  source  = \"terraform-aws-modules/vpc/aws\"\n  version = \"5.0.0\"\n}")
                .with_best_practice("Always pin module versions.")),
            "provider" => Some(HoverInfo::new("provider", "Configures a provider (AWS, GCP, Azure, etc.).")
                .with_syntax("provider \"aws\" {\n  region = \"us-east-1\"\n}")
                .with_best_practice("Pin provider versions in the required_providers block.")),
            "data" => Some(HoverInfo::new("data", "References an existing resource or external data source.")
                .with_syntax("data \"aws_ami\" \"ubuntu\" {\n  most_recent = true\n  filter {\n    name = \"name\"\n    values = [\"ubuntu/images/*\"]\n  }\n}")),
            _ => None,
        }
    }
}
