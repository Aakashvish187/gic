use crate::language_engine::EngineDiagnostic;

const COMMON_TYPOS: &[(&str, &str)] = &[
    ("teh", "the"),
    ("reccomend", "recommend"),
    ("recieve", "receive"),
    ("seperate", "separate"),
    ("occured", "occurred"),
    ("untill", "until"),
    ("acheive", "achieve"),
    ("definate", "definite"),
    ("sucess", "success"),
    ("fuction", "function"),
    ("enviornment", "environment"),
    ("paramater", "parameter"),
    ("impliment", "implement"),
    ("alot", "a lot"),
];

pub fn check_spelling(content: &str, lang: &str) -> Vec<EngineDiagnostic> {
    let mut diagnostics = Vec::new();

    for (row_idx, line) in content.lines().enumerate() {
        // Very simplistic check: only check lines that look like comments
        let is_comment = match lang {
            "yaml" | "bash" | "dockerfile" | "python" | "ruby" => line.trim_start().starts_with('#'),
            "rust" | "c" | "cpp" | "js" | "ts" | "java" | "go" | "csharp" => line.trim_start().starts_with("//"),
            "html" | "xml" => line.trim_start().starts_with("<!--"),
            _ => true, // Check everything in unknown text files
        };

        if !is_comment {
            continue;
        }

        let lower_line = line.to_lowercase();

        for &(typo, correction) in COMMON_TYPOS {
            let mut start = 0;
            while let Some(idx) = lower_line[start..].find(typo) {
                // Check word boundaries
                let is_start_boundary = start + idx == 0 || !lower_line.as_bytes()[start + idx - 1].is_ascii_alphabetic();
                let end_idx = start + idx + typo.len();
                let is_end_boundary = end_idx == lower_line.len() || !lower_line.as_bytes()[end_idx].is_ascii_alphabetic();

                if is_start_boundary && is_end_boundary {
                    let char_col = line[..start + idx].chars().count();
                    diagnostics.push(
                        EngineDiagnostic::warning(
                            row_idx,
                            char_col,
                            format!("Typo: '{}'. Did you mean '{}'?", typo, correction),
                            "spellcheck",
                        )
                        .with_length(typo.len())
                        .with_code("SP001")
                    );
                }
                start += idx + typo.len();
            }
        }
    }

    diagnostics
}
