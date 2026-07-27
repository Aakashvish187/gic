//! Parser registry for managing dynamic language parsers.

use crate::parser::language::LanguageId;
use crate::parser::parser_trait::LanguageParser;
use crate::parser::parsers::{
    BashParser, DockerfileParser, IniParser, JsonParser, MarkdownParser, PlainTextParser,
    TerraformParser, TomlParser, XmlParser, YamlParser,
};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Thread-safe registry mapping `LanguageId` to parser instances.
#[derive(Debug, Clone)]
pub struct ParserRegistry {
    parsers: Arc<RwLock<HashMap<LanguageId, Arc<dyn LanguageParser>>>>,
}

impl ParserRegistry {
    /// Creates an empty parser registry.
    pub fn empty() -> Self {
        Self {
            parsers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Creates a registry pre-populated with all V1 standard language parsers.
    pub fn default_registry() -> Self {
        let registry = Self::empty();
        registry.register(Arc::new(YamlParser::new()));
        registry.register(Arc::new(DockerfileParser::new()));
        registry.register(Arc::new(TerraformParser::new()));
        registry.register(Arc::new(BashParser::new()));
        registry.register(Arc::new(JsonParser::new()));
        registry.register(Arc::new(TomlParser::new()));
        registry.register(Arc::new(MarkdownParser::new()));
        registry.register(Arc::new(IniParser::new()));
        registry.register(Arc::new(XmlParser::new()));
        registry.register(Arc::new(PlainTextParser::new()));
        registry
    }

    /// Registers a new parser implementation for its target language.
    pub fn register(&self, parser: Arc<dyn LanguageParser>) {
        let mut map = self.parsers.write().unwrap();
        map.insert(parser.language(), parser);
    }

    /// Retrieves the registered parser for a given language.
    pub fn get(&self, language: &LanguageId) -> Option<Arc<dyn LanguageParser>> {
        let map = self.parsers.read().unwrap();
        map.get(language).cloned()
    }

    /// Checks if a parser is registered for the specified language.
    pub fn contains(&self, language: &LanguageId) -> bool {
        let map = self.parsers.read().unwrap();
        map.contains_key(language)
    }
}

impl Default for ParserRegistry {
    fn default() -> Self {
        Self::default_registry()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_registration() {
        let registry = ParserRegistry::default_registry();
        assert!(registry.contains(&LanguageId::Yaml));
        assert!(registry.contains(&LanguageId::Dockerfile));
        assert!(registry.contains(&LanguageId::Terraform));
        assert!(registry.contains(&LanguageId::Bash));
        assert!(registry.contains(&LanguageId::Json));
        assert!(registry.contains(&LanguageId::Toml));
        assert!(registry.contains(&LanguageId::Markdown));
        assert!(registry.contains(&LanguageId::Ini));
        assert!(registry.contains(&LanguageId::Xml));
        assert!(registry.contains(&LanguageId::PlainText));
    }
}
