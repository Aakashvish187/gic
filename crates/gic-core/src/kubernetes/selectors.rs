//! Kubernetes LabelSelector Matching and Evaluation Engine.
//!
//! Evaluates `matchLabels` maps and `matchExpressions` rules against target `LabelMap`s to
//! verify resource relationships (Service -> Pod, Deployment -> ReplicaSet -> Pod, NetworkPolicy -> Pod).

use std::collections::HashMap;

use crate::kubernetes::labels::LabelMap;

/// Operator used in a requirement match expression.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SelectorOperator {
    In,
    NotIn,
    Exists,
    DoesNotExist,
}

/// Requirement expression rule (e.g. `{ key: "tier", operator: "In", values: ["frontend", "web"] }`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectorRequirement {
    /// Target label key.
    pub key: String,
    /// Operator matching constraint.
    pub operator: SelectorOperator,
    /// Candidate values for `In` / `NotIn` operators.
    pub values: Vec<String>,
}

/// Kubernetes `LabelSelector` structure.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LabelSelector {
    /// Key-value equality requirements (`matchLabels`).
    pub match_labels: HashMap<String, String>,
    /// Advanced set-based requirements (`matchExpressions`).
    pub match_expressions: Vec<SelectorRequirement>,
}

impl LabelSelector {
    /// Constructs a simple `LabelSelector` with `matchLabels`.
    pub fn from_match_labels(match_labels: HashMap<String, String>) -> Self {
        Self {
            match_labels,
            match_expressions: Vec::new(),
        }
    }

    /// Evaluates whether this selector matches a given target `LabelMap`.
    pub fn matches(&self, target_labels: &LabelMap) -> bool {
        // 1. Check matchLabels equality requirements
        for (k, v) in &self.match_labels {
            match target_labels.get(k) {
                Some(target_val) if target_val == v => {}
                _ => return false,
            }
        }

        // 2. Check matchExpressions requirements
        for req in &self.match_expressions {
            let has_key = target_labels.contains_key(&req.key);
            let target_val = target_labels.get(&req.key);

            match req.operator {
                SelectorOperator::In => {
                    if let Some(v) = target_val {
                        if !req.values.contains(v) {
                            return false;
                        }
                    } else {
                        return false;
                    }
                }
                SelectorOperator::NotIn => {
                    if let Some(v) = target_val {
                        if req.values.contains(v) {
                            return false;
                        }
                    }
                }
                SelectorOperator::Exists => {
                    if !has_key {
                        return false;
                    }
                }
                SelectorOperator::DoesNotExist => {
                    if has_key {
                        return false;
                    }
                }
            }
        }

        true
    }

    /// Returns true if the selector specifies no matching constraints.
    pub fn is_empty(&self) -> bool {
        self.match_labels.is_empty() && self.match_expressions.is_empty()
    }
}
