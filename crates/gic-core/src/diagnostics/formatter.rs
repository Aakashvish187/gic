//! Diagnostic output formatting traits and implementations.

use crate::diagnostics::diagnostic::Diagnostic;
use crate::diagnostics::errors::DiagnosticResult;

/// Trait implemented by diagnostic output formatters (CLI, JSON, TUI).
pub trait DiagnosticFormatter: Send + Sync {
    /// Formats a single diagnostic into a string representation.
    fn format(&self, diagnostic: &Diagnostic) -> DiagnosticResult<String>;

    /// Formats a list of diagnostics into a consolidated output report.
    fn format_all(&self, diagnostics: &[Diagnostic]) -> DiagnosticResult<String> {
        let mut buffer = String::new();
        for (i, diag) in diagnostics.iter().enumerate() {
            if i > 0 {
                buffer.push('\n');
            }
            buffer.push_str(&self.format(diag)?);
        }
        Ok(buffer)
    }
}

/// Plain text formatter producing concise line-oriented outputs.
#[derive(Debug, Default, Clone, Copy)]
pub struct PlainTextFormatter;

impl DiagnosticFormatter for PlainTextFormatter {
    fn format(&self, diagnostic: &Diagnostic) -> DiagnosticResult<String> {
        Ok(format!(
            "{}:{}: [{}] [{}] {}",
            diagnostic.line,
            diagnostic.column,
            diagnostic.severity.tag(),
            diagnostic.rule_name,
            diagnostic.message
        ))
    }
}

/// JSON formatter for machine consumption / LSP integration.
#[derive(Debug, Default, Clone, Copy)]
pub struct JsonFormatter;

impl DiagnosticFormatter for JsonFormatter {
    fn format(&self, diagnostic: &Diagnostic) -> DiagnosticResult<String> {
        serde_json::to_string(diagnostic)
            .map_err(|e| crate::diagnostics::errors::DiagnosticError::SystemError(e.to_string()))
    }

    fn format_all(&self, diagnostics: &[Diagnostic]) -> DiagnosticResult<String> {
        serde_json::to_string_pretty(diagnostics)
            .map_err(|e| crate::diagnostics::errors::DiagnosticError::SystemError(e.to_string()))
    }
}

/// Rich terminal formatter with ASCII/Unicode symbols for CLI displays.
#[derive(Debug, Default, Clone, Copy)]
pub struct PrettyTerminalFormatter;

impl DiagnosticFormatter for PrettyTerminalFormatter {
    fn format(&self, diagnostic: &Diagnostic) -> DiagnosticResult<String> {
        let mut out = format!(
            "{} {} [{}] Line {}, Col {}: {}\n  └─ Rule: {}",
            diagnostic.severity.symbol(),
            diagnostic.severity.tag(),
            diagnostic.language,
            diagnostic.line,
            diagnostic.column,
            diagnostic.message,
            diagnostic.rule_name
        );

        if let Some(desc) = &diagnostic.description {
            out.push_str(&format!("\n  └─ Info: {}", desc));
        }

        if let Some(link) = &diagnostic.documentation_link {
            out.push_str(&format!("\n  └─ Docs: {}", link));
        }

        if !diagnostic.quick_fixes.is_empty() {
            out.push_str(&format!(
                "\n  └─ Quick Fixes ({} available):",
                diagnostic.quick_fixes.len()
            ));
            for qf in &diagnostic.quick_fixes {
                out.push_str(&format!("\n      • {}", qf.title));
            }
        }

        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostics::position::DiagnosticPosition;
    use crate::diagnostics::range::DiagnosticRange;
    use crate::diagnostics::severity::DiagnosticLevel;
    use crate::parser::language::LanguageId;

    #[test]
    fn test_formatters() {
        let p1 = DiagnosticPosition::new(1, 4, 3);
        let range = DiagnosticRange::new(p1, p1);
        let diag = Diagnostic::new(
            DiagnosticLevel::Error,
            "Syntax error found",
            range,
            "YamlValidSyntax",
            LanguageId::Yaml,
        );

        let plain = PlainTextFormatter.format(&diag).unwrap();
        assert_eq!(plain, "1:4: [ERROR] [YamlValidSyntax] Syntax error found");

        let json = JsonFormatter.format(&diag).unwrap();
        assert!(json.contains("YamlValidSyntax"));

        let pretty = PrettyTerminalFormatter.format(&diag).unwrap();
        assert!(pretty.contains("✖ ERROR"));
    }
}
