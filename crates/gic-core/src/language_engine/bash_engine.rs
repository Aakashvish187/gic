//! # Bash / Shell Script Language Engine

use super::{Completion, CompletionKind, EngineDiagnostic, HoverInfo, LanguageEngine};

pub struct BashEngine;

impl BashEngine {
    pub fn new() -> Self {
        Self
    }
}

impl LanguageEngine for BashEngine {
    fn name(&self) -> &'static str {
        "Shell Script"
    }
    fn id(&self) -> &'static str {
        "bash"
    }

    fn diagnostics(&self, content: &str) -> Vec<EngineDiagnostic> {
        let mut diagnostics = Vec::new();

        for (row, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            // Check for unquoted variables that might cause word splitting
            // Simple heuristic: $VAR outside of quotes
            if trimmed.contains('$') && !trimmed.contains('"') {
                // Very simplified check
                let mut in_single_quote = false;
                for (i, ch) in trimmed.char_indices() {
                    if ch == '\'' {
                        in_single_quote = !in_single_quote;
                    }
                    if ch == '$' && !in_single_quote && i + 1 < trimmed.len() {
                        let next = trimmed.chars().nth(i + 1).unwrap_or(' ');
                        if next.is_alphabetic() || next == '{' {
                            // Check if this $ is inside double quotes
                            let before = &trimmed[..i];
                            let dq_count = before.chars().filter(|&c| c == '"').count();
                            if dq_count % 2 == 0 {
                                diagnostics.push(
                                    EngineDiagnostic::hint(row, i,
                                        "Unquoted variable. Use \"$VAR\" to prevent word splitting.",
                                        "bash")
                                        .with_code("SH001")
                                );
                                break; // One per line is enough
                            }
                        }
                    }
                }
            }

            // Check for common mistakes
            if trimmed.starts_with("cd ") && !trimmed.contains("||") && !trimmed.contains("&&") {
                diagnostics.push(
                    EngineDiagnostic::hint(
                        row,
                        0,
                        "'cd' without error handling. Use 'cd dir || exit 1'.",
                        "bash",
                    )
                    .with_code("SH002"),
                );
            }

            // Check for 'rm -rf /' or similar dangerous commands
            if trimmed.contains("rm -rf /")
                && !trimmed.contains("rm -rf /$")
                && !trimmed.contains("rm -rf /\"")
            {
                diagnostics.push(
                    EngineDiagnostic::error(
                        row,
                        0,
                        "DANGEROUS: 'rm -rf /' will destroy the entire filesystem!",
                        "bash",
                    )
                    .with_code("SH003"),
                );
            }

            // Check for 'eval' usage
            if trimmed.starts_with("eval ") || trimmed.contains(" eval ") {
                diagnostics.push(
                    EngineDiagnostic::warning(
                        row,
                        trimmed.find("eval").unwrap_or(0),
                        "'eval' is dangerous and can execute arbitrary code. Avoid if possible.",
                        "bash",
                    )
                    .with_code("SH004"),
                );
            }
        }

        // Check for shebang
        if let Some(first_line) = content.lines().next() {
            if !first_line.starts_with("#!") {
                diagnostics.push(
                    EngineDiagnostic::hint(
                        0,
                        0,
                        "Missing shebang line. Add '#!/usr/bin/env bash' at the top.",
                        "bash",
                    )
                    .with_code("SH005"),
                );
            }
        }

        // Check for 'set -e' or 'set -euo pipefail'
        let has_strict_mode = content.lines().any(|l| {
            let t = l.trim();
            t.starts_with("set -e") || t.starts_with("set -o errexit")
        });
        if !has_strict_mode && content.lines().count() > 5 {
            diagnostics.push(
                EngineDiagnostic::hint(
                    0,
                    0,
                    "Consider adding 'set -euo pipefail' for safer script execution.",
                    "bash",
                )
                .with_code("SH006"),
            );
        }

        diagnostics
    }

    fn completions(&self, _content: &str, _row: usize, _col: usize) -> Vec<Completion> {
        vec![
            Completion::new(
                "if",
                "if [ condition ]; then\n  \nfi",
                CompletionKind::Snippet,
            )
            .with_detail("If statement"),
            Completion::new(
                "for",
                "for item in list; do\n  \ndone",
                CompletionKind::Snippet,
            )
            .with_detail("For loop"),
            Completion::new(
                "while",
                "while [ condition ]; do\n  \ndone",
                CompletionKind::Snippet,
            )
            .with_detail("While loop"),
            Completion::new(
                "function",
                "function name() {\n  \n}",
                CompletionKind::Snippet,
            )
            .with_detail("Function definition"),
            Completion::new(
                "case",
                "case $var in\n  pattern)\n    ;;\nesac",
                CompletionKind::Snippet,
            )
            .with_detail("Case statement"),
            Completion::new(
                "#!/usr/bin/env bash",
                "#!/usr/bin/env bash\nset -euo pipefail\n",
                CompletionKind::Snippet,
            )
            .with_detail("Safe shebang"),
        ]
    }

    fn hover(&self, content: &str, row: usize, _col: usize) -> Option<HoverInfo> {
        let line = content.lines().nth(row)?;
        let trimmed = line.trim();
        let first_word = trimmed.split_whitespace().next()?;

        match first_word {
            "set" => Some(HoverInfo::new("set", "Configures shell options.")
                .with_syntax("set -euo pipefail")
                .with_best_practice("-e: exit on error, -u: error on undefined vars, -o pipefail: catch pipe failures")),
            "export" => Some(HoverInfo::new("export", "Makes a variable available to child processes.")
                .with_syntax("export MY_VAR=\"value\"")),
            "if" => Some(HoverInfo::new("if", "Conditional execution.")
                .with_syntax("if [ condition ]; then\n  commands\nfi")),
            "for" => Some(HoverInfo::new("for", "Loop over a list of items.")
                .with_syntax("for item in list; do\n  echo \"$item\"\ndone")),
            _ => None,
        }
    }

    fn smart_enter(&self, line: &str) -> Option<String> {
        let trimmed = line.trim();
        if trimmed.starts_with("#!/bin/bash") || trimmed.starts_with("#!/usr/bin/env bash") {
            Some("set -euo pipefail\n\n".to_string())
        } else {
            None
        }
    }

    fn template_expansion(&self, keyword: &str) -> Option<String> {
        match keyword.to_lowercase().as_str() {
            "for" => Some("for item in list; do\n  █\ndone".to_string()),
            "if" => Some("if [ condition ]; then\n  █\nfi".to_string()),
            "while" => Some("while [ condition ]; do\n  █\ndone".to_string()),
            "function" => Some("function name() {\n  █\n}".to_string()),
            "case" => Some("case $var in\n  pattern)\n    █\n    ;;\nesac".to_string()),
            _ => None,
        }
    }
}
