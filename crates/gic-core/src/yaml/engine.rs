//! Central YAML Intelligence Engine.
//!
//! Unified façade coordinating parsing, validation, formatting, diagnostic generation,
//! code folding, schema evaluation, completion, hover documentation, and incremental caching.

use std::sync::Arc;

use crate::diagnostics::diagnostic::Diagnostic;
use crate::yaml::cache::YamlCache;
use crate::yaml::completion::{CompletionContext, CompletionItem, YamlCompletionEngine};
use crate::yaml::diagnostics::convert_yaml_diagnostics;
use crate::yaml::errors::YamlResult;
use crate::yaml::folding::{FoldingRange, YamlFoldingEngine};
use crate::yaml::formatter::{YamlFormatter, YamlFormatterOptions};
use crate::yaml::hover::{HoverContext, HoverInfo, YamlHoverEngine};
use crate::yaml::parser::{YamlAST, YamlParser, YamlParserOptions};
use crate::yaml::schema::YamlSchemaRegistry;
use crate::yaml::validator::{YamlValidationDiagnostic, YamlValidator, YamlValidatorOptions};

/// Options controlling `YamlEngine` configuration.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct YamlEngineOptions {
    /// Parser configuration.
    pub parser_options: YamlParserOptions,
    /// Validator configuration.
    pub validator_options: YamlValidatorOptions,
    /// Formatter configuration.
    pub formatter_options: YamlFormatterOptions,
    /// Maximum cache entries limit.
    pub cache_capacity: usize,
}

/// Unified YAML Intelligence Engine facade.
pub struct YamlEngine {
    options: YamlEngineOptions,
    validator: YamlValidator,
    formatter: YamlFormatter,
    folding_engine: YamlFoldingEngine,
    schema_registry: YamlSchemaRegistry,
    completion_engine: YamlCompletionEngine,
    hover_engine: YamlHoverEngine,
    cache: Arc<YamlCache>,
}

impl Default for YamlEngine {
    fn default() -> Self {
        Self::new(YamlEngineOptions {
            cache_capacity: 100,
            ..Default::default()
        })
    }
}

impl YamlEngine {
    /// Creates a new `YamlEngine` with specified configuration options.
    pub fn new(options: YamlEngineOptions) -> Self {
        let validator = YamlValidator::new(options.validator_options.clone());
        let formatter = YamlFormatter::new(options.formatter_options.clone());
        let folding_engine = YamlFoldingEngine::new();
        let schema_registry = YamlSchemaRegistry::new();
        let completion_engine = YamlCompletionEngine::new();
        let hover_engine = YamlHoverEngine::new();
        let cache = Arc::new(YamlCache::new(options.cache_capacity));

        Self {
            options,
            validator,
            formatter,
            folding_engine,
            schema_registry,
            completion_engine,
            hover_engine,
            cache,
        }
    }

    /// Parses raw YAML source into `YamlAST`.
    pub fn parse(&self, source: &str) -> YamlResult<YamlAST> {
        let mut parser = YamlParser::with_options(self.options.parser_options.clone());
        parser.parse(source)
    }

    /// Validates raw YAML source code and returns both internal and central `Diagnostic` items.
    pub fn validate(&self, source: &str) -> (Vec<YamlValidationDiagnostic>, Vec<Diagnostic>) {
        if let Some(entry) = self.cache.get(source) {
            let core_diags = convert_yaml_diagnostics(entry.diagnostics.clone());
            return (entry.diagnostics, core_diags);
        }

        let mut internal_diags = self.validator.validate_source(source);

        // Also run active schema checks if AST parses
        if let Ok(ast) = self.parse(source) {
            let schema_diags = self.schema_registry.validate_active(&ast);
            internal_diags.extend(schema_diags);

            let folding_ranges = self.folding_engine.compute_folding_ranges(&ast);
            self.cache
                .put(source, ast, internal_diags.clone(), None, folding_ranges);
        }

        let core_diags = convert_yaml_diagnostics(internal_diags.clone());
        (internal_diags, core_diags)
    }

    /// Formats raw YAML source code.
    pub fn format(&self, source: &str) -> YamlResult<String> {
        self.formatter.format(source)
    }

    /// Computes code folding ranges for YAML source.
    pub fn compute_folding_ranges(&self, source: &str) -> YamlResult<Vec<FoldingRange>> {
        if let Some(entry) = self.cache.get(source) {
            return Ok(entry.folding_ranges);
        }

        let ast = self.parse(source)?;
        Ok(self.folding_engine.compute_folding_ranges(&ast))
    }

    /// Queries completion items for position in source.
    pub fn autocomplete(&self, ctx: &CompletionContext, source: &str) -> Vec<CompletionItem> {
        let ast = self.parse(source).ok();
        self.completion_engine.complete(ctx, ast.as_ref())
    }

    /// Queries hover information for position in source.
    pub fn hover(&self, ctx: &HoverContext, source: &str) -> Option<HoverInfo> {
        let ast = self.parse(source).ok();
        self.hover_engine.hover(ctx, ast.as_ref())
    }

    /// Access reference to the schema registry for registering custom schemas.
    pub fn schema_registry_mut(&mut self) -> &mut YamlSchemaRegistry {
        &mut self.schema_registry
    }

    /// Access reference to the cache instance.
    pub fn cache(&self) -> &YamlCache {
        &self.cache
    }
}
