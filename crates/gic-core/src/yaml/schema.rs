//! YAML Schema Abstraction and Extension Architecture.
//!
//! Provides extensible trait interfaces, property contracts, and schema registries
//! designed for future schema validation backends (Kubernetes, Docker Compose, Ansible,
//! Helm, GitHub Actions, GitLab CI, Azure Pipelines, CloudFormation).

use std::collections::HashMap;
use std::sync::Arc;

use crate::yaml::parser::YamlAST;
use crate::yaml::validator::YamlValidationDiagnostic;

/// Supported schema domain classifications.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SupportedSchema {
    #[default]
    GenericYaml,
    Kubernetes,
    DockerCompose,
    Ansible,
    GitHubActions,
    Helm,
    Kustomize,
    GitLabCI,
    AzurePipelines,
    CloudFormation,
}

impl SupportedSchema {
    /// Returns human-readable label for schema target.
    pub fn label(&self) -> &'static str {
        match self {
            SupportedSchema::GenericYaml => "Generic YAML",
            SupportedSchema::Kubernetes => "Kubernetes Resource Spec",
            SupportedSchema::DockerCompose => "Docker Compose File",
            SupportedSchema::Ansible => "Ansible Playbook / Task",
            SupportedSchema::GitHubActions => "GitHub Actions Workflow",
            SupportedSchema::Helm => "Helm Chart Values / Chart.yaml",
            SupportedSchema::Kustomize => "Kustomization File",
            SupportedSchema::GitLabCI => "GitLab CI Configuration",
            SupportedSchema::AzurePipelines => "Azure Pipelines Workflow",
            SupportedSchema::CloudFormation => "AWS CloudFormation Template",
        }
    }
}

/// Primitive or structural data type defined in a YAML schema property.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SchemaDataType {
    String,
    Integer,
    Float,
    Boolean,
    Array(Box<SchemaDataType>),
    Object(HashMap<String, SchemaDataType>),
    Any,
}

/// Schema property definition for key/value structure validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchemaProperty {
    /// Property key name.
    pub name: String,
    /// Detailed description or documentation string.
    pub description: String,
    /// Data type specification.
    pub data_type: SchemaDataType,
    /// True if required in the object scope.
    pub required: bool,
}

/// Metadata contract for a complete JSON/YAML Schema document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchemaDefinition {
    /// Unique schema identifier URI or slug.
    pub id: String,
    /// Display name.
    pub name: String,
    /// Schema version tag.
    pub version: String,
    /// Target domain.
    pub target: SupportedSchema,
    /// Root object properties mapped by key name.
    pub properties: HashMap<String, SchemaProperty>,
}

/// Core trait interface for all YAML schema validators.
pub trait YamlSchema: Send + Sync {
    /// Unique schema identifier.
    fn id(&self) -> &str;

    /// Schema display name.
    fn name(&self) -> &str;

    /// Schema target domain.
    fn target(&self) -> SupportedSchema;

    /// Validates a parsed `YamlAST` against this schema contract.
    fn validate(&self, ast: &YamlAST) -> Vec<YamlValidationDiagnostic>;
}

/// Default generic schema implementation validating basic structural validity.
#[derive(Debug, Clone, Default)]
pub struct GenericYamlSchema;

impl GenericYamlSchema {
    pub fn new() -> Self {
        Self
    }
}

impl YamlSchema for GenericYamlSchema {
    fn id(&self) -> &str {
        "generic-yaml-schema"
    }

    fn name(&self) -> &str {
        "Generic YAML Schema"
    }

    fn target(&self) -> SupportedSchema {
        SupportedSchema::GenericYaml
    }

    fn validate(&self, _ast: &YamlAST) -> Vec<YamlValidationDiagnostic> {
        // Generic schema imposes no domain-specific constraints
        Vec::new()
    }
}

/// Central registry for managing active YAML schemas.
#[derive(Clone)]
pub struct YamlSchemaRegistry {
    schemas: HashMap<String, Arc<dyn YamlSchema>>,
    active_schema_id: Option<String>,
}

impl Default for YamlSchemaRegistry {
    fn default() -> Self {
        let mut registry = Self {
            schemas: HashMap::new(),
            active_schema_id: None,
        };
        let generic = Arc::new(GenericYamlSchema::new());
        registry.register(generic.clone());
        registry.set_active(generic.id());
        registry
    }
}

impl YamlSchemaRegistry {
    /// Creates a new schema registry with the default generic schema registered.
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a new schema.
    pub fn register(&mut self, schema: Arc<dyn YamlSchema>) {
        self.schemas.insert(schema.id().to_string(), schema);
    }

    /// Sets the active schema by ID.
    pub fn set_active(&mut self, id: &str) -> bool {
        if self.schemas.contains_key(id) {
            self.active_schema_id = Some(id.to_string());
            true
        } else {
            false
        }
    }

    /// Returns the currently active schema.
    pub fn active_schema(&self) -> Option<Arc<dyn YamlSchema>> {
        self.active_schema_id
            .as_ref()
            .and_then(|id| self.schemas.get(id).cloned())
    }

    /// Validates an AST using the active schema.
    pub fn validate_active(&self, ast: &YamlAST) -> Vec<YamlValidationDiagnostic> {
        if let Some(schema) = self.active_schema() {
            schema.validate(ast)
        } else {
            Vec::new()
        }
    }
}
