//! YAML Intelligence Engine module for GIC (General Infrastructure Console).
//!
//! Provides production-grade YAML parsing, syntax and semantic validation, code formatting with
//! comment preservation, code folding, diagnostic quick-fixes, schema abstraction, hover,
//! autocomplete interfaces, and high-performance result caching.

#![forbid(unsafe_code)]

pub mod aliases;
pub mod anchors;
pub mod cache;
pub mod comments;
pub mod completion;
pub mod diagnostics;
pub mod duplicate_keys;
pub mod engine;
pub mod errors;
pub mod folding;
pub mod formatter;
pub mod hover;
pub mod indentation;
pub mod parser;
pub mod schema;
pub mod validator;

#[cfg(test)]
pub mod tests;

pub use aliases::{AliasIssue, AliasIssueKind, AliasResolver};
pub use anchors::{AnchorExtractor, AnchorInfo, AnchorRegistry};
pub use cache::{YamlCache, YamlCacheEntry, YamlCacheMetrics};
pub use comments::{CommentHandler, CommentPlacement};
pub use completion::{
    CompletionContext, CompletionItem, CompletionKind, CompletionProvider, YamlCompletionEngine,
};
pub use diagnostics::{convert_yaml_diagnostic, convert_yaml_diagnostics};
pub use duplicate_keys::{DuplicateKeyDetector, DuplicateKeyIssue};
pub use engine::{YamlEngine, YamlEngineOptions};
pub use errors::{YamlError, YamlResult};
pub use folding::{FoldingKind, FoldingRange, YamlFoldingEngine};
pub use formatter::{LineEnding, YamlFormatter, YamlFormatterOptions};
pub use hover::{HoverContext, HoverInfo, HoverProvider, YamlHoverEngine};
pub use indentation::{
    IndentChar, IndentationAnalyzer, IndentationIssue, IndentationIssueKind, IndentationReport,
};
pub use parser::{
    AliasReference, AnchorDefinition, Position, Span, Token, TokenKind, YamlAST, YamlComment,
    YamlDocument, YamlKey, YamlMapping, YamlNode, YamlPair, YamlParser, YamlParserOptions,
    YamlScalar, YamlScalarStyle, YamlSequence, YamlValue,
};
pub use schema::{
    GenericYamlSchema, SchemaDataType, SchemaDefinition, SchemaProperty, SupportedSchema,
    YamlSchema, YamlSchemaRegistry,
};
pub use validator::{
    YamlQuickFix, YamlSeverity, YamlValidationDiagnostic, YamlValidator, YamlValidatorOptions,
};
