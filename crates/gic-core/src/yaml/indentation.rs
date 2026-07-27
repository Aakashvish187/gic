//! YAML Indentation Analysis Engine.
//!
//! Inspects YAML source code line by line for indentation anomalies such as tab usage,
//! unaligned nesting levels, and non-standard spacing steps.

use crate::yaml::parser::{Position, Span};

/// Summary report of indentation analysis.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndentationReport {
    /// List of detected indentation issues.
    pub issues: Vec<IndentationIssue>,
    /// Indentation character used throughout the document (spaces vs tabs).
    pub indent_char: IndentChar,
    /// Primary indentation step size (commonly 2 spaces).
    pub detected_step: usize,
}

/// Type of indentation character detected.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum IndentChar {
    #[default]
    Spaces,
    Tabs,
    Mixed,
}

/// Description of an indentation flaw.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndentationIssue {
    /// Error code or issue category.
    pub kind: IndentationIssueKind,
    /// Human-readable explanation.
    pub message: String,
    /// Line number (1-indexed).
    pub line: usize,
    /// Expected indentation width in characters.
    pub expected: usize,
    /// Actual found indentation width.
    pub found: usize,
    /// Exact span of the leading indentation whitespace.
    pub span: Span,
}

/// Specific category of indentation defect.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IndentationIssueKind {
    /// Use of tab characters (`\t`) in indentation.
    TabCharacter,
    /// Mixed tabs and spaces in leading line whitespace.
    MixedTabsAndSpaces,
    /// Non-uniform indentation step size.
    InconsistentStep,
    /// Unexpected increase in indentation without parent context.
    UnexpectedIndent,
    /// Trailing space at end of line.
    TrailingWhitespace,
}

/// Analyzer for YAML line-by-line whitespace and block nesting levels.
#[derive(Debug, Clone)]
pub struct IndentationAnalyzer {
    expected_step: usize,
}

impl Default for IndentationAnalyzer {
    fn default() -> Self {
        Self::new(2)
    }
}

impl IndentationAnalyzer {
    /// Constructs a new analyzer configured with an expected step size (default 2).
    pub fn new(expected_step: usize) -> Self {
        Self { expected_step }
    }

    /// Analyzes raw YAML source text and returns an `IndentationReport`.
    pub fn analyze(&self, source: &str) -> IndentationReport {
        let mut issues = Vec::new();
        let mut has_spaces = false;
        let mut has_tabs = false;
        let mut prev_indent = 0;

        for (line_idx, line) in source.lines().enumerate() {
            let line_number = line_idx + 1;

            if line.trim().is_empty() || line.trim_start().starts_with('#') {
                continue;
            }

            let mut leading_spaces = 0;
            let mut leading_tabs = 0;
            let mut char_count = 0;

            for ch in line.chars() {
                if ch == ' ' {
                    leading_spaces += 1;
                    char_count += 1;
                } else if ch == '\t' {
                    leading_tabs += 1;
                    char_count += 1;
                } else {
                    break;
                }
            }

            if leading_spaces > 0 {
                has_spaces = true;
            }
            if leading_tabs > 0 {
                has_tabs = true;
            }

            let span = Span::new(
                Position::new(line_number, 1, 0),
                Position::new(line_number, char_count + 1, char_count),
            );

            // 1. Tab character detection
            if leading_tabs > 0 {
                if leading_spaces > 0 {
                    issues.push(IndentationIssue {
                        kind: IndentationIssueKind::MixedTabsAndSpaces,
                        message: "Line contains mixed tabs and spaces in indentation".to_string(),
                        line: line_number,
                        expected: leading_spaces + (leading_tabs * self.expected_step),
                        found: char_count,
                        span,
                    });
                } else {
                    issues.push(IndentationIssue {
                        kind: IndentationIssueKind::TabCharacter,
                        message: "Tabs are prohibited for YAML indentation; use spaces instead"
                            .to_string(),
                        line: line_number,
                        expected: char_count * self.expected_step,
                        found: char_count,
                        span,
                    });
                }
            }

            // 2. Indentation step check
            let current_indent = char_count;
            if current_indent > 0 && current_indent % self.expected_step != 0 {
                issues.push(IndentationIssue {
                    kind: IndentationIssueKind::InconsistentStep,
                    message: format!(
                        "Indentation width of {} is not a multiple of configured step size {}",
                        current_indent, self.expected_step
                    ),
                    line: line_number,
                    expected: (current_indent / self.expected_step) * self.expected_step,
                    found: current_indent,
                    span,
                });
            }

            // 3. Unexpected sudden indent jump (> expected step jump)
            if current_indent > prev_indent + (self.expected_step * 2) && prev_indent > 0 {
                issues.push(IndentationIssue {
                    kind: IndentationIssueKind::UnexpectedIndent,
                    message: format!(
                        "Unexpected indentation jump from {} to {}",
                        prev_indent, current_indent
                    ),
                    line: line_number,
                    expected: prev_indent + self.expected_step,
                    found: current_indent,
                    span,
                });
            }

            prev_indent = current_indent;
        }

        let indent_char = match (has_spaces, has_tabs) {
            (true, false) => IndentChar::Spaces,
            (false, true) => IndentChar::Tabs,
            (true, true) => IndentChar::Mixed,
            (false, false) => IndentChar::Spaces,
        };

        IndentationReport {
            issues,
            indent_char,
            detected_step: self.expected_step,
        }
    }
}
