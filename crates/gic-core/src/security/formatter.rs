//! Security report formatters for JSON, Markdown, and PlainText output.

use crate::security::reporting::SecurityReport;
use serde_json;

/// Security report output format.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReportFormat {
    Json,
    Markdown,
    PlainText,
}

/// Security report formatter capable of rendering findings to multiple output formats.
#[derive(Debug, Clone, Default)]
pub struct SecurityReportFormatter;

impl SecurityReportFormatter {
    pub fn new() -> Self {
        Self
    }

    /// Formats a security report into the requested output format.
    pub fn format(&self, report: &SecurityReport, format: ReportFormat) -> String {
        match format {
            ReportFormat::Json => self.format_json(report),
            ReportFormat::Markdown => self.format_markdown(report),
            ReportFormat::PlainText => self.format_plain(report),
        }
    }

    fn format_json(&self, report: &SecurityReport) -> String {
        serde_json::to_string_pretty(report).unwrap_or_else(|_| "{}".to_string())
    }

    fn format_markdown(&self, report: &SecurityReport) -> String {
        let mut output = String::new();
        output.push_str("# GIC Security Report\n\n");
        output.push_str(&format!("**Risk Score:** {}\n\n", report.risk_score));
        output.push_str("## Severity Summary\n\n");
        output.push_str(&format!("| Severity | Count |\n|----------|-------|\n"));
        output.push_str(&format!(
            "| 🔥 Critical | {} |\n",
            report.severity_counts.critical
        ));
        output.push_str(&format!("| 🚨 High | {} |\n", report.severity_counts.high));
        output.push_str(&format!(
            "| ⚠ Medium | {} |\n",
            report.severity_counts.medium
        ));
        output.push_str(&format!("| ⚡ Low | {} |\n", report.severity_counts.low));
        output.push_str(&format!(
            "| ℹ Information | {} |\n",
            report.severity_counts.information
        ));
        output.push_str("\n## Findings\n\n");
        for finding in &report.findings {
            output.push_str(&format!(
                "- **[{}]** {} — *{}*\n  > {}\n  > **Remediation:** {}\n\n",
                finding.severity,
                finding.title,
                finding.category,
                finding.description,
                finding.remediation,
            ));
        }
        output
    }

    fn format_plain(&self, report: &SecurityReport) -> String {
        let mut output = String::new();
        output.push_str("=== GIC SECURITY REPORT ===\n\n");
        output.push_str(&format!("Risk Score: {}\n\n", report.risk_score));
        output.push_str(&format!(
            "Critical: {} | High: {} | Medium: {} | Low: {} | Info: {}\n\n",
            report.severity_counts.critical,
            report.severity_counts.high,
            report.severity_counts.medium,
            report.severity_counts.low,
            report.severity_counts.information,
        ));
        output.push_str("FINDINGS:\n");
        for finding in &report.findings {
            output.push_str(&format!(
                "[{}] {} ({})\n  {}\n  Remediation: {}\n\n",
                finding.severity,
                finding.title,
                finding.category,
                finding.description,
                finding.remediation,
            ));
        }
        output
    }
}
