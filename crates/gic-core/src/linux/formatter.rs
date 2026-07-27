//! Shell Script and Config Code Formatter.

#[derive(Debug, Clone, Default)]
pub struct LinuxFormatter;

impl LinuxFormatter {
    pub fn new() -> Self {
        Self
    }

    pub fn format(&self, source: &str) -> String {
        let mut result = String::with_capacity(source.len());
        for line in source.lines() {
            result.push_str(line.trim_end());
            result.push('\n');
        }
        result
    }
}
