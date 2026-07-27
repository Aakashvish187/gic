//! High-level ParsingEngine façade coordinating detection, registration, caching, and tree generation.

use crate::parser::cache::{CacheMetrics, ParseCache};
use crate::parser::errors::ParseError;
use crate::parser::language::{LanguageDetector, LanguageId};
use crate::parser::loader::ParserLoader;
use crate::parser::position::TextChange;
use crate::parser::registry::ParserRegistry;
use crate::parser::tree::SyntaxTree;
use crate::parser::utils::hash_source;
use std::path::Path;

/// The central entry point for the Universal Language Parsing Engine.
#[derive(Debug, Clone)]
pub struct ParsingEngine {
    detector: LanguageDetector,
    registry: ParserRegistry,
    loader: ParserLoader,
    cache: ParseCache,
}

impl ParsingEngine {
    /// Creates a new `ParsingEngine` with default settings and pre-loaded V1 parsers.
    pub fn new() -> Self {
        let registry = ParserRegistry::default_registry();
        let loader = ParserLoader::new(registry.clone());
        let cache = ParseCache::new(100);
        let detector = LanguageDetector::new();

        Self {
            detector,
            registry,
            loader,
            cache,
        }
    }

    /// Automatically detects language and parses source text into a `SyntaxTree`.
    pub fn parse_source(
        &self,
        path: Option<&Path>,
        source: &str,
        override_lang: Option<LanguageId>,
    ) -> Result<SyntaxTree, ParseError> {
        let first_line = source.lines().next();
        let lang = self.detector.detect(path, first_line, override_lang);
        let source_hash = hash_source(source);

        let cache_key = path
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| format!("anon_{:x}", source_hash));

        if let Some(cached_tree) = self.cache.get(&cache_key, source_hash) {
            return Ok(cached_tree);
        }

        let parser = self.loader.load(&lang)?;
        let tree = parser.parse(source, None)?;

        self.cache.insert(cache_key, tree.clone());
        Ok(tree)
    }

    /// Incrementally updates an existing parse tree given a text change delta.
    pub fn parse_incremental(
        &self,
        key: &str,
        path: Option<&Path>,
        source: &str,
        change: &TextChange,
        override_lang: Option<LanguageId>,
    ) -> Result<SyntaxTree, ParseError> {
        let _first_line = source.lines().next();
        let lang = self.detector.detect(path, _first_line, override_lang);
        let _source_hash = hash_source(source);

        let old_tree = self.cache.get(key, 0); // Check for existing cached tree ignoring hash
        let parser = self.loader.load(&lang)?;

        let tree = if let Some(old) = old_tree {
            parser.parse_incremental(source, change, &old)?
        } else {
            parser.parse(source, None)?
        };

        self.cache.insert(key.to_string(), tree.clone());
        Ok(tree)
    }

    /// Returns a reference to the internal parser registry.
    pub fn registry(&self) -> &ParserRegistry {
        &self.registry
    }

    /// Exposes auto-detection logic directly.
    pub fn detect_language(
        &self,
        path: Option<&Path>,
        first_line_or_shebang: Option<&str>,
        override_lang: Option<LanguageId>,
    ) -> LanguageId {
        self.detector
            .detect(path, first_line_or_shebang, override_lang)
    }

    /// Clears the parse tree cache.
    pub fn clear_cache(&self) {
        self.cache.clear();
    }

    /// Returns cache performance metrics.
    pub fn cache_metrics(&self) -> CacheMetrics {
        self.cache.metrics()
    }
}

impl Default for ParsingEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_full_workflow() {
        let engine = ParsingEngine::new();
        let yaml_source = "apiVersion: v1\nkind: Pod\nmetadata:\n  name: test-pod\n";

        let tree = engine
            .parse_source(Some(Path::new("pod.yaml")), yaml_source, None)
            .expect("Parsing should succeed");

        assert_eq!(tree.language, LanguageId::Yaml);
        assert!(!tree.root.children.is_empty());

        // Second call should hit cache
        let cached = engine
            .parse_source(Some(Path::new("pod.yaml")), yaml_source, None)
            .unwrap();
        assert_eq!(cached, tree);
        assert_eq!(engine.cache_metrics().hits, 1);
    }
}
