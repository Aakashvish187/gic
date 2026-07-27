//! Unified Linux Intelligence Engine Façade.

use crate::diagnostics::diagnostic::Diagnostic;
use crate::linux::cache::LinuxCache;
use crate::linux::diagnostics::convert_linux_diagnostics;
use crate::linux::formatter::LinuxFormatter;
use crate::linux::validator::{LinuxDiagnostic, LinuxValidator};
use crate::parser::language::LanguageId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinuxEngineOptions {
    pub enable_cache: bool,
    pub cache_capacity: usize,
}

impl Default for LinuxEngineOptions {
    fn default() -> Self {
        Self {
            enable_cache: true,
            cache_capacity: 256,
        }
    }
}

#[derive(Debug)]
pub struct LinuxEngine {
    validator: LinuxValidator,
    cache: LinuxCache,
    formatter: LinuxFormatter,
    options: LinuxEngineOptions,
}

impl Default for LinuxEngine {
    fn default() -> Self {
        Self::new(LinuxEngineOptions::default())
    }
}

impl LinuxEngine {
    pub fn new(options: LinuxEngineOptions) -> Self {
        Self {
            validator: LinuxValidator::new(),
            cache: LinuxCache::new(options.cache_capacity),
            formatter: LinuxFormatter::new(),
            options,
        }
    }

    pub fn cache(&self) -> &LinuxCache {
        &self.cache
    }

    pub fn validate_bash(&self, source: &str) -> (Vec<LinuxDiagnostic>, Vec<Diagnostic>) {
        self.validate_with_cache(source, LanguageId::Bash, |v, s| v.validate_bash_script(s))
    }

    pub fn validate_systemd(&self, source: &str) -> (Vec<LinuxDiagnostic>, Vec<Diagnostic>) {
        self.validate_with_cache(source, LanguageId::Ini, |v, s| v.validate_systemd(s))
    }

    fn validate_with_cache<F>(
        &self,
        source: &str,
        lang: LanguageId,
        validate_fn: F,
    ) -> (Vec<LinuxDiagnostic>, Vec<Diagnostic>)
    where
        F: FnOnce(&LinuxValidator, &str) -> Vec<LinuxDiagnostic>,
    {
        let hash = LinuxCache::compute_hash(source);

        if self.options.enable_cache {
            if let Some(cached) = self.cache.get(hash) {
                let gic_diags = convert_linux_diagnostics(&cached, lang);
                return (cached, gic_diags);
            }
        }

        let diags = validate_fn(&self.validator, source);

        if self.options.enable_cache {
            self.cache.insert(hash, diags.clone());
        }

        let gic_diags = convert_linux_diagnostics(&diags, lang);
        (diags, gic_diags)
    }

    pub fn format(&self, source: &str) -> String {
        self.formatter.format(source)
    }

    pub fn get_validator(&self) -> &LinuxValidator {
        &self.validator
    }
}
