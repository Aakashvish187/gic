//! # gic-core
//! Core domain entities, interfaces, error types, and traits for GIC.

pub mod buffer;
pub mod config;
pub mod diagnostics;
pub mod docker;
pub mod engine;
pub mod error;
pub mod event;
pub mod fs;
pub mod git;
pub mod kubernetes;
pub mod linux;
pub mod metadata;
pub mod parser;
pub mod rules;
pub mod search;
pub mod shutdown;
pub mod terraform;
pub mod yaml;

pub use metadata::{AboutInfo, AboutProvider, DefaultAboutProvider, ProjectMetadata};

pub use git::{
    BlameEngine, BlameHunk, BranchEngine, BranchKind, CachedRepoData, DiffEngine, DiffHunk,
    DiffLine, DiffOptions, DiscoveredRepo, EditorDecorations, FileDiff, FileStatus,
    FileStatusEntry, GitBranch, GitCache, GitCommit, GitDetector, GitDiagnostics, GitEngine,
    GitError, GitIgnoreEngine, GitLogger, GitMetrics, GitRepository, GitResult, GutterDecoration,
    HistoryEngine, LineChangeKind, LineDecoration, RepositoryStatus, StatusBarGitInfo,
    StatusEngine,
};

pub use docker::{
    DockerCache, DockerDiagnostic, DockerEngine, DockerEngineOptions, DockerError, DockerResult,
    DockerValidator, DockerfileAST, DockerfileParser,
};
pub use kubernetes::{
    K8sCache, K8sDiagnostic, K8sEngine, K8sEngineOptions, K8sError, K8sResource,
    K8sResourceDetector, K8sResourceKind, K8sResult, K8sValidator,
};
pub use linux::{
    BashAST, CommandInvocation, LinuxCache, LinuxDiagnostic, LinuxEngine, LinuxEngineOptions,
    LinuxError, LinuxFormatter, LinuxResult, LinuxValidator, Shebang, ShellKind,
};
pub use terraform::{
    HclAttribute, HclBlock, KnownProvider, ModuleCall, OutputDeclaration, ProviderConfiguration,
    TerraformAST, TerraformCache, TerraformDiagnostic, TerraformEngine, TerraformEngineOptions,
    TerraformError, TerraformFormatter, TerraformFormatterOptions, TerraformParser,
    TerraformResource, TerraformResult, TerraformSecurityAnalyzer, TerraformSeverity,
    TerraformValidator, VariableDeclaration,
};
pub use yaml::{
    YamlAST, YamlCache, YamlDocument, YamlEngine, YamlEngineOptions, YamlError, YamlFormatter,
    YamlFormatterOptions, YamlNode, YamlParser, YamlResult, YamlValidator, YamlValidatorOptions,
};

pub use buffer::{
    BufferCommand, BufferError, BufferOperations, ClipboardContentType, CommandGroup, Cursor,
    CursorPosition, InternalClipboard, Selection, SelectionMode, TextBuffer, UndoRedoHistory,
};
pub use config::{AppConfig, LogConfig, UIConfig};
pub use engine::{EngineMetrics, EngineState};
pub use error::GicError;
pub use event::{
    InputEvent, KeyCode, KeyInput, KeyModifiers, MouseAction, MouseButton, MouseInput,
};
pub use fs::{
    Document, DocumentContent, FileMetadata, FileReader, FileSystemManager, RecentFileEntry,
    RecentFilesManager,
};
pub use parser::{
    CacheMetrics, Diagnostic, DiagnosticSeverity, LanguageDetector, LanguageId, LanguageParser,
    LanguageSpec, NodeKind, ParseCache, ParseError, ParserLoader, ParserRegistry, ParsingEngine,
    Position, SymbolInformation, SyntaxNode, SyntaxTree, TextChange, TextRange, Token, TokenKind,
    TokenStream, TreeSitterBackend,
};
pub use search::{
    HighlightEngine, HighlightKind, HighlightRange, HorspoolMatcher, MatchNavigator, MatchRange,
    PatternMatcher, PlaceholderRegexEngine, RegexEngine, ReplaceEngine, ReplaceResult, SearchCache,
    SearchDirection, SearchEngine, SearchError, SearchHistory, SearchMatch, SearchOptions,
    SearchQuery, SearchScope, SearchStatistics,
};
pub use shutdown::{ShutdownReason, ShutdownSignal};

pub use diagnostics::{
    DiagnosticCache, DiagnosticLevel, DiagnosticPosition, DiagnosticRange, DiagnosticResult,
    GenericRuleValidator, JsonFormatter, PlainTextFormatter, PrettyTerminalFormatter, QuickFix,
    QuickFixKind, Rule, RuleCategory, RuleConfig, RuleMetadata, RulePriority, RuleRegistry,
    TextEdit, ValidationContext, ValidationEngine, ValidationMetrics, Validator, ValidatorRegistry,
};
