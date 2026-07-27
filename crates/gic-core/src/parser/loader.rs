//! Dynamic parser loading and fallback resolution.

use crate::parser::errors::ParseError;
use crate::parser::language::LanguageId;
use crate::parser::parser_trait::LanguageParser;
use crate::parser::parsers::PlainTextParser;
use crate::parser::registry::ParserRegistry;
use std::sync::Arc;

/// Responsible for fetching or lazily instantiating language parsers.
#[derive(Debug, Clone)]
pub struct ParserLoader {
    registry: ParserRegistry,
}

impl ParserLoader {
    /// Creates a new `ParserLoader` backed by the given registry.
    pub fn new(registry: ParserRegistry) -> Self {
        Self { registry }
    }

    /// Loads a parser for the specified language ID, falling back to Plain Text if not found.
    pub fn load(&self, language: &LanguageId) -> Result<Arc<dyn LanguageParser>, ParseError> {
        if let Some(parser) = self.registry.get(language) {
            Ok(parser)
        } else if let Some(fallback) = self.registry.get(&LanguageId::PlainText) {
            Ok(fallback)
        } else {
            Ok(Arc::new(PlainTextParser::new()))
        }
    }
}
