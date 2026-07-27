//! Unit tests for YAML Schema Architecture.

use crate::yaml::schema::{SupportedSchema, YamlSchemaRegistry};

#[test]
fn test_schema_registry_registration_and_active_schema() {
    let registry = YamlSchemaRegistry::new();
    let active = registry.active_schema();

    assert!(active.is_some());
    assert_eq!(active.unwrap().id(), "generic-yaml-schema");
}

#[test]
fn test_supported_schema_labels() {
    assert_eq!(
        SupportedSchema::Kubernetes.label(),
        "Kubernetes Resource Spec"
    );
    assert_eq!(
        SupportedSchema::DockerCompose.label(),
        "Docker Compose File"
    );
    assert_eq!(SupportedSchema::Ansible.label(), "Ansible Playbook / Task");
    assert_eq!(
        SupportedSchema::GitHubActions.label(),
        "GitHub Actions Workflow"
    );
}
