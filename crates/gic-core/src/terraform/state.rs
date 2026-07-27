//! Terraform State File (`.tfstate`) Data Model and Drift Interface.
//!
//! Models serialized state schema, tracks managed resource addresses, outputs,
//! serial numbers, and prepares drift detection contracts.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Deserialized `.tfstate` document model.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct TerraformState {
    /// Schema format version (typically `4`).
    pub version: u32,
    /// Terraform binary version that generated state.
    pub terraform_version: String,
    /// Monotonically increasing state version sequence.
    pub serial: u64,
    /// Unique lineage ID of the state workspace.
    pub lineage: String,
    /// State outputs map.
    pub outputs: HashMap<String, StateOutput>,
    /// Managed and data resources recorded in state.
    pub resources: Vec<StateResource>,
}

/// Recorded state output.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct StateOutput {
    /// Primitive value type identifier.
    pub value_type: String,
    /// Sensitive marker.
    pub sensitive: bool,
}

/// Recorded state resource.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct StateResource {
    /// Module path hierarchy.
    pub module: Option<String>,
    /// Resource mode (`managed` or `data`).
    pub mode: String,
    /// Provider type.
    pub resource_type: String,
    /// Local resource name.
    pub name: String,
    /// Provider namespace.
    pub provider_name: String,
}

/// Drift detection status result interface.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DriftReport {
    /// Resources present in HCL but missing in state.
    pub missing_in_state: Vec<String>,
    /// Resources present in state but removed from HCL.
    pub orphaned_in_state: Vec<String>,
}
