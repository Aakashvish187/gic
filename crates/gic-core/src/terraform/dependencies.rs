//! Terraform Resource Dependency Graph Builder and Cycle Detector.
//!
//! Maps implicit and explicit (`depends_on`) resource edges, constructs Directed Acyclic Graphs (DAG),
//! and detects circular dependencies.

use std::collections::{HashMap, HashSet, VecDeque};

use crate::terraform::resources::TerraformResource;

/// Dependency edge between source and target resources.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DependencyEdge {
    /// Source referencing resource address (e.g. `"aws_instance.web"`).
    pub source: String,
    /// Target referenced resource address (e.g. `"aws_security_group.sg"`).
    pub target: String,
    /// True if dependency is explicit (`depends_on`).
    pub is_explicit: bool,
}

/// Dependency graph representation.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DependencyGraph {
    /// Resource addresses node list.
    pub nodes: HashSet<String>,
    /// Directed adjacency edges: source -> list of targets.
    pub adjacency: HashMap<String, Vec<DependencyEdge>>,
}

/// Dependency graph builder and cycle detection service.
#[derive(Debug, Clone, Default)]
pub struct DependencyAnalyzer;

impl DependencyAnalyzer {
    /// Creates a new DependencyAnalyzer.
    pub fn new() -> Self {
        Self
    }

    /// Builds a `DependencyGraph` from a set of `TerraformResource` items.
    pub fn build_graph(&self, resources: &[TerraformResource]) -> DependencyGraph {
        let mut nodes = HashSet::new();
        let mut adjacency: HashMap<String, Vec<DependencyEdge>> = HashMap::new();

        for res in resources {
            nodes.insert(res.address.clone());

            // 1. Explicit depends_on edges
            for dep in &res.depends_on {
                nodes.insert(dep.clone());
                adjacency
                    .entry(res.address.clone())
                    .or_default()
                    .push(DependencyEdge {
                        source: res.address.clone(),
                        target: dep.clone(),
                        is_explicit: true,
                    });
            }

            // 2. Implicit reference edges from attributes
            for attr_val in res.attributes.values() {
                for target_addr in extract_referenced_addresses(attr_val) {
                    if target_addr != res.address {
                        nodes.insert(target_addr.clone());
                        adjacency
                            .entry(res.address.clone())
                            .or_default()
                            .push(DependencyEdge {
                                source: res.address.clone(),
                                target: target_addr,
                                is_explicit: false,
                            });
                    }
                }
            }
        }

        DependencyGraph { nodes, adjacency }
    }

    /// Detects circular dependency cycles using Kahn's topological sort algorithm.
    pub fn find_cycles(&self, graph: &DependencyGraph) -> Vec<Vec<String>> {
        let mut in_degree: HashMap<String, usize> = HashMap::new();
        for node in &graph.nodes {
            in_degree.insert(node.clone(), 0);
        }

        for edges in graph.adjacency.values() {
            for edge in edges {
                *in_degree.entry(edge.target.clone()).or_default() += 1;
            }
        }

        let mut queue = VecDeque::new();
        for (node, &deg) in &in_degree {
            if deg == 0 {
                queue.push_back(node.clone());
            }
        }

        let mut visited_count = 0;
        while let Some(u) = queue.pop_front() {
            visited_count += 1;
            if let Some(edges) = graph.adjacency.get(&u) {
                for edge in edges {
                    if let Some(deg) = in_degree.get_mut(&edge.target) {
                        *deg = deg.saturating_sub(1);
                        if *deg == 0 {
                            queue.push_back(edge.target.clone());
                        }
                    }
                }
            }
        }

        if visited_count < graph.nodes.len() {
            let cycle_nodes: Vec<String> = in_degree
                .into_iter()
                .filter(|(_, deg)| *deg > 0)
                .map(|(node, _)| node)
                .collect();
            if !cycle_nodes.is_empty() {
                return vec![cycle_nodes];
            }
        }

        Vec::new()
    }
}

fn extract_referenced_addresses(val: &str) -> Vec<String> {
    let mut addrs = Vec::new();
    for token in val.split(|c: char| {
        c.is_whitespace() || c == '"' || c == '\'' || c == ',' || c == '{' || c == '}'
    }) {
        let clean = token.trim_start_matches("${").trim_end_matches('}');
        let parts: Vec<&str> = clean.split('.').collect();
        if parts.len() >= 2 && !parts[0].contains(' ') && !parts[1].contains(' ')
            && is_valid_identifier(parts[0]) && is_valid_identifier(parts[1]) {
                addrs.push(format!("{}.{}", parts[0], parts[1]));
            }
    }
    addrs
}

fn is_valid_identifier(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| c.is_alphanumeric() || c == '_')
}
