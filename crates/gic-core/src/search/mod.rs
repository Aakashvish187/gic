//! # Search Engine Module for GIC Editor
//! Professional, high-performance, non-blocking search and replace engine.

pub mod cache;
pub mod engine;
pub mod errors;
pub mod highlights;
pub mod history;
pub mod matcher;
pub mod navigator;
pub mod options;
pub mod query;
pub mod regex;
pub mod replace;
pub mod statistics;

pub use cache::SearchCache;
pub use engine::SearchEngine;
pub use errors::SearchError;
pub use highlights::{HighlightEngine, HighlightKind, HighlightRange};
pub use history::SearchHistory;
pub use matcher::{HorspoolMatcher, MatchRange, PatternMatcher, SearchMatch};
pub use navigator::MatchNavigator;
pub use options::{SearchDirection, SearchMode, SearchOptions, SearchScope};
pub use query::SearchQuery;
pub use regex::{PlaceholderRegexEngine, RegexEngine};
pub use replace::{ReplaceEngine, ReplaceResult};
pub use statistics::SearchStatistics;
