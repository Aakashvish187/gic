//! Unified Terraform Intelligence Engine Façade.
//!
//! Provides single-entrypoint API for validating Terraform configuration files (`.tf`, `.tfvars`, `.hcl`),
//! code formatting, incremental validation caching, and GIC diagnostics conversion.

use crate::diagnostics::diagnostic::Diagnostic;
use crate::parser::language::LanguageId;
use crate::terraform::cache::TerraformCache;
use crate::terraform::diagnostics::convert_terraform_diagnostics;
use crate::terraform::formatter::{TerraformFormatter, TerraformFormatterOptions};
use crate::terraform::validator::{TerraformDiagnostic, TerraformValidator};

/// Options for configuring `TerraformEngine`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerraformEngineOptions {
    /// Enable incremental LRU caching.
    pub enable_cache: bool,
    /// Maximum cache capacity.
    pub cache_capacity: usize,
    /// Formatter options.
    pub formatter_options: TerraformFormatterOptions,
}

impl Default for TerraformEngineOptions {
    fn default() -> Self {
        Self {
            enable_cache: true,
            cache_capacity: 256,
            formatter_options: TerraformFormatterOptions::default(),
        }
    }
}

/// Unified Terraform Intelligence Engine.
#[derive(Debug)]
pub struct TerraformEngine {
    validator: TerraformValidator,
    cache: TerraformCache,
    formatter: TerraformFormatter,
    options: TerraformEngineOptions,
}

impl Default for TerraformEngine {
    fn default() -> Self {
        Self::new(TerraformEngineOptions::default())
    }
}

impl TerraformEngine {
    /// Constructs a new TerraformEngine with custom options.
    pub fn new(options: TerraformEngineOptions) -> Self {
        let capacity = options.cache_capacity;
        let formatter = TerraformFormatter::with_options(options.formatter_options.clone());

        Self {
            validator: TerraformValidator::new(),
            cache: TerraformCache::new(capacity),
            formatter,
            options,
        }
    }

    /// Returns reference to internal cache.
    pub fn cache(&self) -> &TerraformCache {
        &self.cache
    }

    /// Validates raw Terraform source code and returns native and GIC diagnostics.
    pub fn validate_source(&self, source: &str) -> (Vec<TerraformDiagnostic>, Vec<Diagnostic>) {
        let hash = TerraformCache::compute_hash(source);

        if self.options.enable_cache {
            if let Some(cached) = self.cache.get(hash) {
                let gic_diags = convert_terraform_diagnostics(&cached, LanguageId::Terraform);
                return (cached, gic_diags);
            }
        }

        let tf_diags = self.validator.validate_source(source);

        if self.options.enable_cache {
            self.cache.insert(hash, tf_diags.clone());
        }

        let gic_diags = convert_terraform_diagnostics(&tf_diags, LanguageId::Terraform);
        (tf_diags, gic_diags)
    }

    /// Formats raw `.tf` source code.
    pub fn format_source(&self, source: &str) -> String {
        self.formatter.format(source)
    }
}
