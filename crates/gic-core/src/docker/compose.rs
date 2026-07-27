//! Docker Compose Detector and Document Structure Engine.
//!
//! Recognizes Docker Compose YAML documents (`services:`, `version:`, `networks:`, `volumes:`, `secrets:`, `configs:`),
//! extracts top-level sections, and prepares Compose AST structures.

use std::collections::HashMap;

use crate::yaml::parser::{Span, YamlAST, YamlMapping, YamlNode, YamlPair, YamlValue};

/// Extracted Docker Compose document model.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ComposeDocument {
    /// Compose file spec version string (e.g. `"3.8"`, `"3.9"`).
    pub version: String,
    /// Service mapping names -> service root node.
    pub services: HashMap<String, YamlNode>,
    /// Top-level network mapping names -> network root node.
    pub networks: HashMap<String, YamlNode>,
    /// Top-level volume mapping names -> volume root node.
    pub volumes: HashMap<String, YamlNode>,
    /// Top-level secret mapping names -> secret root node.
    pub secrets: HashMap<String, YamlNode>,
    /// Top-level config mapping names -> config root node.
    pub configs: HashMap<String, YamlNode>,
    /// Span covering entire document.
    pub span: Span,
    /// Reference to underlying root node.
    pub root_node: Option<YamlNode>,
}

/// Detector for identifying Docker Compose manifests.
#[derive(Debug, Clone, Default)]
pub struct ComposeDetector;

impl ComposeDetector {
    /// Creates a new ComposeDetector.
    pub fn new() -> Self {
        Self
    }

    /// Checks if a `YamlAST` represents a Docker Compose document.
    pub fn is_compose_document(&self, ast: &YamlAST) -> bool {
        for doc in &ast.documents {
            if let Some(ref root) = doc.root {
                if let YamlValue::Mapping(ref map) = root.value {
                    let has_services = map.pairs.iter().any(|p| p.key.value == "services");
                    let has_version = map.pairs.iter().any(|p| p.key.value == "version");
                    if has_services || has_version {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Extracts `ComposeDocument` structures from a `YamlAST`.
    pub fn parse_compose(&self, ast: &YamlAST) -> Vec<ComposeDocument> {
        let mut compose_docs = Vec::new();

        for doc in &ast.documents {
            if let Some(ref root) = doc.root {
                if let YamlValue::Mapping(ref map) = root.value {
                    let has_services = map.pairs.iter().any(|p| p.key.value == "services");
                    let has_version = map.pairs.iter().any(|p| p.key.value == "version");

                    if has_services || has_version {
                        let parsed = parse_single_compose_doc(root, map, doc.span);
                        compose_docs.push(parsed);
                    }
                }
            }
        }

        compose_docs
    }
}

fn parse_single_compose_doc(root: &YamlNode, map: &YamlMapping, span: Span) -> ComposeDocument {
    let mut version = String::new();
    let mut services = HashMap::new();
    let mut networks = HashMap::new();
    let mut volumes = HashMap::new();
    let mut secrets = HashMap::new();
    let mut configs = HashMap::new();

    let mut current_section = "";
    let mut current_item_name = String::new();
    let mut current_item_pairs = Vec::new();
    let mut active_property = String::new();

    for pair in &map.pairs {
        let k = pair.key.value.as_str();

        if k == "version" {
            if let YamlValue::Scalar(ref s) = pair.value.value {
                version = s.value.clone();
            }
            continue;
        }

        let is_top_section_keyword = matches!(
            k,
            "services" | "networks" | "volumes" | "secrets" | "configs"
        );

        if is_top_section_keyword && current_item_name.is_empty() {
            flush_current_item(
                root,
                current_section,
                &current_item_name,
                &current_item_pairs,
                &mut services,
                &mut networks,
                &mut volumes,
                &mut secrets,
                &mut configs,
            );
            current_section = k;
            current_item_name.clear();
            current_item_pairs.clear();
            active_property.clear();
        } else if current_section == "services" {
            if is_service_property(k) {
                active_property = k.to_string();
                current_item_pairs.push(pair.clone());
            } else if is_multi_item_property(&active_property) {
                current_item_pairs.push(pair.clone());
            } else {
                flush_current_item(
                    root,
                    current_section,
                    &current_item_name,
                    &current_item_pairs,
                    &mut services,
                    &mut networks,
                    &mut volumes,
                    &mut secrets,
                    &mut configs,
                );
                current_item_name = k.to_string();
                current_item_pairs.clear();
                active_property.clear();
            }
        } else if !current_section.is_empty() {
            if is_top_section_keyword && current_section != k {
                flush_current_item(
                    root,
                    current_section,
                    &current_item_name,
                    &current_item_pairs,
                    &mut services,
                    &mut networks,
                    &mut volumes,
                    &mut secrets,
                    &mut configs,
                );
                current_section = k;
                current_item_name.clear();
                current_item_pairs.clear();
                active_property.clear();
            } else if is_resource_property(k) {
                active_property = k.to_string();
                current_item_pairs.push(pair.clone());
            } else if is_multi_item_property(&active_property) {
                current_item_pairs.push(pair.clone());
            } else {
                flush_current_item(
                    root,
                    current_section,
                    &current_item_name,
                    &current_item_pairs,
                    &mut services,
                    &mut networks,
                    &mut volumes,
                    &mut secrets,
                    &mut configs,
                );
                current_item_name = k.to_string();
                current_item_pairs.clear();
                active_property.clear();
            }
        }
    }

    flush_current_item(
        root,
        current_section,
        &current_item_name,
        &current_item_pairs,
        &mut services,
        &mut networks,
        &mut volumes,
        &mut secrets,
        &mut configs,
    );

    ComposeDocument {
        version,
        services,
        networks,
        volumes,
        secrets,
        configs,
        span,
        root_node: Some(root.clone()),
    }
}

fn is_service_property(k: &str) -> bool {
    matches!(
        k,
        "image"
            | "build"
            | "ports"
            | "environment"
            | "env_file"
            | "volumes"
            | "networks"
            | "depends_on"
            | "healthcheck"
            | "restart"
            | "secrets"
            | "configs"
            | "profiles"
            | "privileged"
            | "cap_add"
            | "cap_drop"
            | "read_only"
            | "network_mode"
            | "command"
            | "entrypoint"
            | "user"
            | "working_dir"
            | "expose"
            | "labels"
    )
}

fn is_multi_item_property(k: &str) -> bool {
    matches!(
        k,
        "ports"
            | "volumes"
            | "networks"
            | "depends_on"
            | "environment"
            | "env_file"
            | "labels"
            | "secrets"
            | "configs"
            | "profiles"
            | "cap_add"
            | "cap_drop"
            | "command"
            | "entrypoint"
            | "expose"
    )
}

fn is_resource_property(k: &str) -> bool {
    matches!(
        k,
        "driver" | "driver_opts" | "external" | "file" | "name" | "labels" | "attachable" | "ipam"
    )
}

fn flush_current_item(
    root: &YamlNode,
    section: &str,
    item_name: &str,
    pairs: &[YamlPair],
    services: &mut HashMap<String, YamlNode>,
    networks: &mut HashMap<String, YamlNode>,
    volumes: &mut HashMap<String, YamlNode>,
    secrets: &mut HashMap<String, YamlNode>,
    configs: &mut HashMap<String, YamlNode>,
) {
    if item_name.is_empty() {
        return;
    }

    let node = YamlNode {
        id: root.id,
        anchor: None,
        value: YamlValue::Mapping(YamlMapping {
            pairs: pairs.to_vec(),
            span: root.span,
        }),
        leading_comments: Vec::new(),
        trailing_comment: None,
        span: root.span,
    };

    match section {
        "services" => {
            services.insert(item_name.to_string(), node);
        }
        "networks" => {
            networks.insert(item_name.to_string(), node);
        }
        "volumes" => {
            volumes.insert(item_name.to_string(), node);
        }
        "secrets" => {
            secrets.insert(item_name.to_string(), node);
        }
        "configs" => {
            configs.insert(item_name.to_string(), node);
        }
        _ => {}
    }
}
