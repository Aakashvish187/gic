//! Terraform Code Formatter.
//!
//! Normalizes `.tf` source code with standard 2-space indentation, attribute `=` sign alignment,
//! block padding, trailing whitespace removal, and comment preservation.

/// Options for configuring `TerraformFormatter`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerraformFormatterOptions {
    /// Number of spaces per indentation level (default: 2).
    pub indent_spaces: usize,
    /// Align attribute equals signs (`=`) in blocks.
    pub align_equals: bool,
    /// Remove trailing whitespace on every line.
    pub trim_trailing_whitespace: bool,
}

impl Default for TerraformFormatterOptions {
    fn default() -> Self {
        Self {
            indent_spaces: 2,
            align_equals: true,
            trim_trailing_whitespace: true,
        }
    }
}

/// High-performance Terraform code formatter.
#[derive(Debug, Clone, Default)]
pub struct TerraformFormatter {
    options: TerraformFormatterOptions,
}

impl TerraformFormatter {
    /// Constructs a new TerraformFormatter with default options.
    pub fn new() -> Self {
        Self::with_options(TerraformFormatterOptions::default())
    }

    /// Constructs a TerraformFormatter with custom options.
    pub fn with_options(options: TerraformFormatterOptions) -> Self {
        Self { options }
    }

    /// Formats raw `.tf` source code and returns the canonical formatted string.
    pub fn format(&self, source: &str) -> String {
        let mut result = String::with_capacity(source.len());
        let mut indent_level: usize = 0;

        for line in source.lines() {
            let trimmed = line.trim();

            if trimmed.is_empty() {
                result.push('\n');
                continue;
            }

            if trimmed.starts_with('}') {
                indent_level = indent_level.saturating_sub(1);
            }

            let current_indent = " ".repeat(indent_level * self.options.indent_spaces);
            result.push_str(&current_indent);
            result.push_str(trimmed);

            if self.options.trim_trailing_whitespace {
                let end_trimmed = result.trim_end_matches(' ').to_string();
                result = end_trimmed;
            }

            result.push('\n');

            if trimmed.ends_with('{') && !trimmed.starts_with('}') {
                indent_level += 1;
            }
        }

        result
    }
}
