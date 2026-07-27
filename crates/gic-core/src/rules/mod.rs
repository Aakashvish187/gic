//! Universal Rule Engine & Diagnostic Framework
//!
//! This module provides the foundation for evaluating rules across all language engines.

pub mod action;
pub mod cache;
pub mod category;
pub mod condition;
pub mod configuration;
pub mod context;
pub mod documentation;
pub mod engine;
pub mod errors;
pub mod evaluator;
pub mod logger;
pub mod matcher;
pub mod metadata;
pub mod metrics;
pub mod priority;
pub mod profile;
pub mod quick_fix;
pub mod registry;
pub mod rule;
pub mod scheduler;
pub mod severity;
pub mod statistics;
pub mod tags;

pub use action::DiagnosticAction;
pub use cache::RuleCache;
pub use category::RuleCategory;
pub use condition::{AndCondition, MatchStringCondition, NotCondition, OrCondition, RuleCondition};
pub use configuration::{RuleConfiguration, WorkspaceSettings};
pub use context::EvaluationContext;
pub use documentation::{DocumentationExample, RuleDocumentation};
pub use engine::UniversalRuleEngine;
pub use errors::{Result, RuleEngineError};
pub use evaluator::RuleEvaluator;
pub use logger::RuleLogger;
pub use matcher::StringMatcher;
pub use metadata::RuleMetadata;
pub use metrics::RuleEngineMetrics;
pub use priority::RulePriority;
pub use profile::RuleProfile;
pub use quick_fix::{QuickFixOperation, RuleQuickFix};
pub use registry::RuleRegistry;
pub use rule::{Rule, RuleAction, RuleContext};
pub use scheduler::RuleScheduler;
pub use severity::RuleSeverity;
pub use statistics::RuleStatistics;
pub use tags::RuleTags;

#[cfg(test)]
mod tests;
