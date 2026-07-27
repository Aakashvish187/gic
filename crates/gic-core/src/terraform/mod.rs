//! Terraform & Infrastructure-as-Code Intelligence Engine for GIC.
//!
//! Provides HCL AST parsing, resource extraction, provider validation, variable and output checks,
//! module reusability analysis, backend and state locking checks, expression interpolation,
//! dependency graph cycle detection, IaC security auditing, networking rules, best practice guidance,
//! formatting, completion/hover contracts, and caching.

pub mod backend;
pub mod best_practices;
pub mod cache;
pub mod completion;
pub mod dependencies;
pub mod diagnostics;
pub mod engine;
pub mod errors;
pub mod formatter;
pub mod hover;
pub mod interpolation;
pub mod modules;
pub mod networking;
pub mod outputs;
pub mod parser;
pub mod providers;
pub mod references;
pub mod resources;
pub mod security;
pub mod state;
pub mod validator;
pub mod variables;

#[cfg(test)]
pub mod tests;

pub use backend::{BackendConfiguration, BackendType, BackendValidator};
pub use best_practices::{BestPracticeRecommendation, BestPracticesAnalyzer};
pub use cache::TerraformCache;
pub use completion::{CompletionKind, TerraformCompleter, TerraformCompletionItem};
pub use dependencies::{DependencyAnalyzer, DependencyEdge, DependencyGraph};
pub use diagnostics::{convert_terraform_diagnostic, convert_terraform_diagnostics};
pub use engine::{TerraformEngine, TerraformEngineOptions};
pub use errors::{TerraformError, TerraformResult};
pub use formatter::{TerraformFormatter, TerraformFormatterOptions};
pub use hover::{HoverDoc, TerraformHoverProvider};
pub use interpolation::{InterpolationAnalyzer, InterpolationExpression, InterpolationKind};
pub use modules::{ModuleCall, ModuleSourceKind, ModuleValidator};
pub use networking::NetworkSecurityAnalyzer;
pub use outputs::{OutputDeclaration, OutputValidator};
pub use parser::{HclAttribute, HclBlock, TerraformAST, TerraformParser};
pub use providers::{KnownProvider, ProviderConfiguration, ProviderValidator};
pub use references::{ReferenceResolver, SymbolTable};
pub use resources::{ResourceExtractor, ResourceMode, TerraformResource};
pub use security::{
    TerraformSecurityAnalyzer, TerraformSecurityFinding, TerraformSecuritySeverity,
};
pub use state::{DriftReport, StateOutput, StateResource, TerraformState};
pub use validator::{TerraformDiagnostic, TerraformSeverity, TerraformValidator};
pub use variables::{VariableDeclaration, VariableValidator};
