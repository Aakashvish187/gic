//! YAML Formatter Engine with Comment Preservation.
//!
//! Formats raw YAML text or AST into clean, standardized code preserving comments,
//! normalizing line endings, trimming trailing whitespace, and enforcing consistent indentation.

use crate::yaml::errors::YamlResult;

/// Configured line ending convention.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LineEnding {
    /// Unix line ending (`\n`).
    #[default]
    Lf,
    /// Windows line ending (`\r\n`).
    Crlf,
}

impl LineEnding {
    /// Returns string slice for line ending.
    pub fn as_str(&self) -> &'static str {
        match self {
            LineEnding::Lf => "\n",
            LineEnding::Crlf => "\r\n",
        }
    }
}

/// Options controlling YAML formatting behavior.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct YamlFormatterOptions {
    /// Number of spaces per indentation level (default 2).
    pub indent_step: usize,
    /// Remove whitespace at end of lines (default true).
    pub trim_trailing_whitespace: bool,
    /// Collapse multiple empty lines into at most one blank line (default true).
    pub normalize_blank_lines: bool,
    /// Preserves `# comment` lines and inline comments (default true).
    pub preserve_comments: bool,
    /// Target line ending style (default LF).
    pub line_ending: LineEnding,
}

impl Default for YamlFormatterOptions {
    fn default() -> Self {
        Self {
            indent_step: 2,
            trim_trailing_whitespace: true,
            normalize_blank_lines: true,
            preserve_comments: true,
            line_ending: LineEnding::Lf,
        }
    }
}

/// YAML Formatter.
#[derive(Debug, Clone)]
pub struct YamlFormatter {
    options: YamlFormatterOptions,
}

impl Default for YamlFormatter {
    fn default() -> Self {
        Self::new(YamlFormatterOptions::default())
    }
}

impl YamlFormatter {
    /// Constructs a new YamlFormatter with given options.
    pub fn new(options: YamlFormatterOptions) -> Self {
        Self { options }
    }

    /// Formats a raw YAML string according to configured formatting rules.
    pub fn format(&self, source: &str) -> YamlResult<String> {
        let eol = self.options.line_ending.as_str();
        let mut formatted_lines = Vec::new();
        let mut consecutive_empty_lines = 0;

        for line in source.lines() {
            let mut processed_line = line.to_string();

            // 1. Convert tab indentation to configured space step
            if processed_line.starts_with('\t') || processed_line.contains('\t') {
                let leading_tabs = processed_line.chars().take_while(|c| *c == '\t').count();
                if leading_tabs > 0 {
                    let spaces = " ".repeat(leading_tabs * self.options.indent_step);
                    processed_line = format!("{}{}", spaces, &processed_line[leading_tabs..]);
                }
            }

            // 2. Trim trailing whitespace if enabled
            if self.options.trim_trailing_whitespace {
                processed_line = processed_line.trim_end().to_string();
            }

            // 3. Normalize blank lines
            if processed_line.is_empty() {
                consecutive_empty_lines += 1;
                if self.options.normalize_blank_lines && consecutive_empty_lines > 1 {
                    continue;
                }
            } else {
                consecutive_empty_lines = 0;
            }

            // 4. Ensure space after colon in mappings (`key:val` -> `key: val`)
            processed_line = format_mapping_colon_spacing(&processed_line);

            formatted_lines.push(processed_line);
        }

        // Join lines with line endings
        let mut result = formatted_lines.join(eol);
        if !result.is_empty() && !result.ends_with(eol) {
            result.push_str(eol);
        }

        Ok(result)
    }
}

/// Utility function ensuring space after colon separator in mapping lines.
fn format_mapping_colon_spacing(line: &str) -> String {
    let trimmed = line.trim_start();
    if trimmed.starts_with('#') {
        return line.to_string();
    }

    if let Some(colon_idx) = line.find(':') {
        let before = &line[..colon_idx];
        let after = &line[colon_idx + 1..];

        // Only add space if after colon is non-empty, not whitespace, not comment, and not newline
        if !after.is_empty()
            && !after.starts_with(' ')
            && !after.starts_with('\t')
            && !after.starts_with('#')
        {
            return format!("{}: {}", before, after);
        }
    }
    line.to_string()
}
