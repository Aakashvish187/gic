//! Unified Docker & Docker Compose Intelligence Engine Façade.
//!
//! Provides single-entrypoint API for validating Dockerfiles and `docker-compose.yml` files,
//! integrating incremental caching, diagnostics conversion, and layer analysis.

use crate::diagnostics::diagnostic::Diagnostic;
use crate::docker::cache::DockerCache;
use crate::docker::diagnostics::convert_docker_diagnostics;
use crate::docker::validator::{DockerDiagnostic, DockerValidator};
use crate::parser::language::LanguageId;
use crate::yaml::parser::YamlParser;

/// Options for configuring the `DockerEngine`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DockerEngineOptions {
    /// Enable incremental caching.
    pub enable_cache: bool,
    /// Maximum cache entries.
    pub cache_capacity: usize,
}

impl Default for DockerEngineOptions {
    fn default() -> Self {
        Self {
            enable_cache: true,
            cache_capacity: 256,
        }
    }
}

/// Unified Docker Intelligence Engine.
#[derive(Debug)]
pub struct DockerEngine {
    validator: DockerValidator,
    cache: DockerCache,
    options: DockerEngineOptions,
}

impl Default for DockerEngine {
    fn default() -> Self {
        Self::new(DockerEngineOptions::default())
    }
}

impl DockerEngine {
    /// Constructs a new DockerEngine with custom options.
    pub fn new(options: DockerEngineOptions) -> Self {
        let capacity = options.cache_capacity;
        Self {
            validator: DockerValidator::new(),
            cache: DockerCache::new(capacity),
            options,
        }
    }

    /// Returns cache reference.
    pub fn cache(&self) -> &DockerCache {
        &self.cache
    }

    /// Validates raw Dockerfile source code and returns both native and GIC diagnostics.
    pub fn validate_dockerfile(&self, source: &str) -> (Vec<DockerDiagnostic>, Vec<Diagnostic>) {
        let hash = DockerCache::compute_hash(source);

        if self.options.enable_cache {
            if let Some(cached) = self.cache.get(hash) {
                let gic_diags = convert_docker_diagnostics(&cached, LanguageId::Dockerfile);
                return (cached, gic_diags);
            }
        }

        let docker_diags = self.validator.validate_dockerfile(source);

        if self.options.enable_cache {
            self.cache.insert(hash, docker_diags.clone());
        }

        let gic_diags = convert_docker_diagnostics(&docker_diags, LanguageId::Dockerfile);
        (docker_diags, gic_diags)
    }

    /// Validates raw Docker Compose YAML source code and returns both native and GIC diagnostics.
    pub fn validate_compose(&self, source: &str) -> (Vec<DockerDiagnostic>, Vec<Diagnostic>) {
        let hash = DockerCache::compute_hash(source);

        if self.options.enable_cache {
            if let Some(cached) = self.cache.get(hash) {
                let gic_diags = convert_docker_diagnostics(&cached, LanguageId::Yaml);
                return (cached, gic_diags);
            }
        }

        let mut yaml_parser = YamlParser::new();
        let docker_diags = match yaml_parser.parse(source) {
            Ok(ast) => self.validator.validate_compose_ast(&ast),
            Err(_) => Vec::new(),
        };

        if self.options.enable_cache {
            self.cache.insert(hash, docker_diags.clone());
        }

        let gic_diags = convert_docker_diagnostics(&docker_diags, LanguageId::Yaml);
        (docker_diags, gic_diags)
    }
}
