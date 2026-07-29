//! Security Intelligence Engine façade — unified public API for all security operations.

use crate::diagnostics::Diagnostic;
use crate::security::analyzer::{EngineDiagnosticSource, SecurityAnalyzer};
use crate::security::cache::SecurityCache;
use crate::security::diagnostics::SecurityDiagnostics;
use crate::security::errors::SecurityResult;
use crate::security::formatter::{ReportFormat, SecurityReportFormatter};
use crate::security::logger::SecurityLogger;
use crate::security::metrics::SecurityMetrics;
use crate::security::policy_registry::PolicyRegistry;
use crate::security::reporting::{SecurityReport, SecurityReporter};
use crate::security::scanner::SecurityScanner;
use std::path::Path;

/// Unified Security Intelligence Engine for GIC.
///
/// Provides a single entry point for:
/// - Content-level secret/credential scanning
/// - Domain diagnostic aggregation (Docker, K8s, Terraform, Linux, etc.)
/// - Compliance policy evaluation
/// - Risk scoring and report generation
/// - Editor-level diagnostic conversion
pub struct SecurityEngine {
    scanner: SecurityScanner,
    analyzer: SecurityAnalyzer,
    reporter: SecurityReporter,
    formatter: SecurityReportFormatter,
    diagnostics: SecurityDiagnostics,
    cache: SecurityCache,
    metrics: SecurityMetrics,
    logger: SecurityLogger,
    policy_registry: PolicyRegistry,
}

impl SecurityEngine {
    /// Creates a new `SecurityEngine` with all sub-systems initialized.
    pub fn new() -> SecurityResult<Self> {
        Ok(Self {
            scanner: SecurityScanner::new()?,
            analyzer: SecurityAnalyzer::new(),
            reporter: SecurityReporter::new(),
            formatter: SecurityReportFormatter::new(),
            diagnostics: SecurityDiagnostics::new(),
            cache: SecurityCache::new(),
            metrics: SecurityMetrics::new(),
            logger: SecurityLogger::new(),
            policy_registry: PolicyRegistry::default(),
        })
    }

    /// Scans a file's content for secrets, credentials, and certificates.
    ///
    /// Returns cached results if the path was scanned previously without changes.
    pub fn scan_file_content<P: AsRef<Path>>(
        &self,
        path: P,
        content: &str,
    ) -> Vec<crate::security::findings::SecurityFinding> {
        let path = path.as_ref().to_path_buf();
        self.logger
            .log_scan_started(path.to_str().unwrap_or("unknown"));
        self.metrics.inc_scans();

        let findings = self.scanner.scan_content(Some(&path), content);
        let count = findings.len();
        self.metrics.inc_findings(count as u64);
        self.cache.put_file_findings(path.clone(), findings.clone());
        self.logger
            .log_scan_complete(path.to_str().unwrap_or("unknown"), count);
        findings
    }

    /// Aggregates security findings from all domain engine diagnostic outputs.
    pub fn aggregate_engine_diagnostics(
        &self,
        sources: Vec<EngineDiagnosticSource<'_>>,
    ) -> Vec<crate::security::findings::SecurityFinding> {
        self.metrics.inc_scans();
        let findings = self.analyzer.analyze_all(sources);
        self.metrics.inc_findings(findings.len() as u64);
        findings
    }

    /// Builds a complete `SecurityReport` from the supplied findings.
    pub fn build_report(
        &self,
        findings: Vec<crate::security::findings::SecurityFinding>,
    ) -> SecurityReport {
        let count = findings.len();
        let report = self.reporter.build_report(findings);
        self.metrics.inc_reports();
        self.logger
            .log_report_built(report.risk_score.value(), count);
        report
    }

    /// Formats a `SecurityReport` in the requested output format.
    pub fn format_report(&self, report: &SecurityReport, format: ReportFormat) -> String {
        self.formatter.format(report, format)
    }

    /// Converts security findings into GIC `Diagnostic` objects for editor integration.
    pub fn to_gic_diagnostics(
        &self,
        findings: &[crate::security::findings::SecurityFinding],
    ) -> Vec<Diagnostic> {
        self.diagnostics.to_diagnostics(findings)
    }

    /// Returns the current operational metrics snapshot.
    pub fn metrics(&self) -> &SecurityMetrics {
        &self.metrics
    }

    /// Returns the policy registry for compliance rule management.
    pub fn policy_registry(&self) -> &PolicyRegistry {
        &self.policy_registry
    }

    /// Returns the cache for direct cache management.
    pub fn cache(&self) -> &SecurityCache {
        &self.cache
    }
}
