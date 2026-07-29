use std::collections::HashMap;
use crate::language_engine::{Completion, CompletionKind, HoverInfo};

/// Represents the data type of a property.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SchemaDataType {
    String,
    Number,
    Boolean,
    Object,
    Array(Box<SchemaDataType>),
    Enum(Vec<String>),
    Suggestions(Vec<String>),
    Any,
}

/// A property definition in a schema.
#[derive(Debug, Clone)]
pub struct SchemaProperty {
    pub name: String,
    pub description: String,
    pub data_type: SchemaDataType,
    pub required: bool,
    pub default_value: Option<String>,
    pub snippet: Option<String>, // For "Generate Child Blocks"
    pub example: Option<String>,
    pub production_recommendation: Option<String>,
    pub security_recommendation: Option<String>,
    pub common_mistakes: Option<String>,
}

impl SchemaProperty {
    pub fn new(name: impl Into<String>, description: impl Into<String>, data_type: SchemaDataType) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            data_type,
            required: false,
            default_value: None,
            snippet: None,
            example: None,
            production_recommendation: None,
            security_recommendation: None,
            common_mistakes: None,
        }
    }

    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }

    pub fn with_default(mut self, default_value: impl Into<String>) -> Self {
        self.default_value = Some(default_value.into());
        self
    }

    pub fn with_snippet(mut self, snippet: impl Into<String>) -> Self {
        self.snippet = Some(snippet.into());
        self
    }

    pub fn with_example(mut self, example: impl Into<String>) -> Self {
        self.example = Some(example.into());
        self
    }
    
    pub fn with_production_recommendation(mut self, rec: impl Into<String>) -> Self {
        self.production_recommendation = Some(rec.into());
        self
    }

    pub fn with_security_recommendation(mut self, rec: impl Into<String>) -> Self {
        self.security_recommendation = Some(rec.into());
        self
    }
    
    pub fn with_common_mistakes(mut self, mistakes: impl Into<String>) -> Self {
        self.common_mistakes = Some(mistakes.into());
        self
    }
}

/// A schema node representing an object (like "Deployment" or "Container").
#[derive(Debug, Clone)]
pub struct SchemaNode {
    pub name: String,
    pub description: String,
    pub properties: HashMap<String, SchemaProperty>,
}

impl SchemaNode {
    pub fn new(name: &str, description: &str) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            properties: HashMap::new(),
        }
    }

    pub fn with_property(mut self, prop: SchemaProperty) -> Self {
        self.properties.insert(prop.name.clone(), prop);
        self
    }
}

/// The main trait for a language schema provider.
pub trait LanguageSchema: Send + Sync {
    /// Returns the schema node for a given contextual path (e.g., ["spec", "template", "spec", "containers"]).
    fn resolve_path(&self, path: &[String], resource_kind: Option<&str>) -> Option<&SchemaNode>;

    /// Returns the property definition for a key within a contextual path.
    fn resolve_property(&self, path: &[String], key: &str, resource_kind: Option<&str>) -> Option<&SchemaProperty> {
        self.resolve_path(path, resource_kind).and_then(|node| node.properties.get(key))
    }

    /// Provides completions for keys at the given path.
    fn key_completions(&self, path: &[String], base_indent: usize, resource_kind: Option<&str>) -> Vec<Completion> {
        let mut completions = Vec::new();
        if let Some(node) = self.resolve_path(path, resource_kind) {
            for (name, prop) in &node.properties {
                let kind = match &prop.data_type {
                    SchemaDataType::Object => CompletionKind::Property,
                    SchemaDataType::Array(_) => CompletionKind::Property,
                    _ => CompletionKind::Value,
                };
                
                // Normal key completion
                completions.push(
                    Completion::new(name, format!("{}: ", name), kind)
                        .with_detail(&prop.description)
                );
                
                // Generate child blocks if snippet exists
                if let Some(snippet) = &prop.snippet {
                    let mut indented_snippet = String::new();
                    let spaces = " ".repeat(base_indent);
                    
                    for (i, line) in snippet.lines().enumerate() {
                        if i == 0 {
                            indented_snippet.push_str(line);
                        } else {
                            indented_snippet.push('\n');
                            indented_snippet.push_str(&spaces);
                            indented_snippet.push_str(line);
                        }
                    }
                    if snippet.ends_with('\n') {
                        indented_snippet.push('\n');
                        indented_snippet.push_str(&spaces); // So cursor ends up at correct indent
                    }

                    completions.push(
                        Completion::new(
                            format!("{} (Template)", name),
                            indented_snippet,
                            CompletionKind::Snippet
                        ).with_detail("Generate full block")
                    );
                }
            }
        }
        completions
    }

    fn value_completions(&self, path: &[String], key: &str, resource_kind: Option<&str>) -> Vec<Completion> {
        let mut completions = Vec::new();
        if let Some(prop) = self.resolve_property(path, key, resource_kind) {
            match &prop.data_type {
                SchemaDataType::Enum(values) | SchemaDataType::Suggestions(values) => {
                    for val in values {
                        completions.push(
                            Completion::new(val.as_str(), val.as_str(), CompletionKind::Value)
                                .with_detail(&prop.description)
                        );
                    }
                }
                SchemaDataType::Boolean => {
                    completions.push(Completion::new("true", "true", CompletionKind::Value));
                    completions.push(Completion::new("false", "false", CompletionKind::Value));
                }
                _ => {}
            }
        }
        completions
    }

    /// Provides hover documentation for a key at a given path.
    fn hover_info(&self, path: &[String], key: &str, resource_kind: Option<&str>) -> Option<HoverInfo> {
        self.resolve_property(path, key, resource_kind).map(|prop| {
            let mut h = HoverInfo::new(&prop.name, &prop.description);
            if let SchemaDataType::Enum(ref values) = prop.data_type {
                h = h.with_syntax(format!("Allowed values: {}", values.join(", ")));
            } else if let SchemaDataType::Suggestions(ref values) = prop.data_type {
                h = h.with_syntax(format!("Common values: {}", values.join(", ")));
            }
            h.example = prop.example.clone();
            h.production_recommendation = prop.production_recommendation.clone();
            h.security_recommendation = prop.security_recommendation.clone();
            h.common_mistakes = prop.common_mistakes.clone();
            h
        })
    }
}
