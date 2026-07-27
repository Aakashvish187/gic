//! Dockerfile Instruction AST Nodes and Enum Definitions.
//!
//! Represents all 18+ standard Dockerfile instructions, their flags (`--from`, `--chown`),
//! raw arguments, line numbers, and byte spans.

use std::fmt;

use crate::yaml::parser::Span;

/// Individual Dockerfile instruction variant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstructionKind {
    /// `FROM image:tag [AS stage]`
    From {
        /// Base image string (e.g., `alpine:3.19`, `golang:1.22-bookworm`).
        image: String,
        /// Optional target platform (`--platform=linux/amd64`).
        platform: Option<String>,
        /// Optional build stage name (`AS builder`).
        stage_alias: Option<String>,
    },
    /// `RUN command`
    Run {
        /// Raw shell or exec form command array.
        command: Vec<String>,
        /// True if exec form JSON array (`RUN ["apt-get", "update"]`).
        is_exec_form: bool,
    },
    /// `COPY [--from=stage] [--chown=user:group] src... dest`
    Copy {
        /// Source paths.
        sources: Vec<String>,
        /// Destination path.
        destination: String,
        /// Optional `--from=<stage>` cross-stage reference.
        from_stage: Option<String>,
        /// Optional `--chown=<owner:group>` ownership flag.
        chown: Option<String>,
    },
    /// `ADD [--chown=user:group] src... dest`
    Add {
        /// Source paths or URLs.
        sources: Vec<String>,
        /// Destination path.
        destination: String,
        /// Optional `--chown=<owner:group>` ownership flag.
        chown: Option<String>,
    },
    /// `CMD ["executable", "param1"]` or `CMD command param1`
    Cmd {
        /// Arguments array.
        arguments: Vec<String>,
        /// True if exec form JSON array.
        is_exec_form: bool,
    },
    /// `ENTRYPOINT ["executable", "param1"]` or `ENTRYPOINT command param1`
    Entrypoint {
        /// Arguments array.
        arguments: Vec<String>,
        /// True if exec form JSON array.
        is_exec_form: bool,
    },
    /// `ENV key=value ...` or `ENV key value`
    Env {
        /// Key-value environment variable pairs.
        pairs: Vec<(String, String)>,
    },
    /// `ARG name[=default_value]`
    Arg {
        /// Argument variable name.
        name: String,
        /// Optional default value.
        default_value: Option<String>,
    },
    /// `EXPOSE port[/protocol] ...`
    Expose {
        /// Ports/protocols array (e.g. `["80/tcp", "443"]`).
        ports: Vec<String>,
    },
    /// `LABEL key=value ...`
    Label {
        /// Key-value metadata label pairs.
        pairs: Vec<(String, String)>,
    },
    /// `USER user[:group]` or `USER uid[:gid]`
    User {
        /// User identifier string.
        user: String,
        /// Optional group identifier string.
        group: Option<String>,
    },
    /// `WORKDIR /path`
    Workdir {
        /// Absolute or relative working directory path.
        path: String,
    },
    /// `HEALTHCHECK [options] CMD command` or `HEALTHCHECK NONE`
    Healthcheck {
        /// True if `HEALTHCHECK NONE`.
        is_none: bool,
        /// Interval string (e.g., `30s`).
        interval: Option<String>,
        /// Timeout string (e.g., `3s`).
        timeout: Option<String>,
        /// Start period string (e.g., `5s`).
        start_period: Option<String>,
        /// Retries count.
        retries: Option<usize>,
        /// Health check command array.
        command: Vec<String>,
    },
    /// `STOPSIGNAL signal`
    Stopsignal {
        /// System call signal string (e.g. `SIGTERM`, `9`).
        signal: String,
    },
    /// `SHELL ["executable", "param1"]`
    Shell {
        /// Executable shell command array.
        shell: Vec<String>,
    },
    /// `VOLUME ["/data"]`
    Volume {
        /// Volume paths array.
        paths: Vec<String>,
    },
    /// `ONBUILD instruction`
    Onbuild {
        /// Raw trigger instruction string.
        trigger: String,
    },
    /// Raw unparsed instruction fallback.
    Raw {
        /// Uppercase keyword.
        keyword: String,
        /// Argument payload string.
        payload: String,
    },
}

impl InstructionKind {
    /// Returns uppercase instruction keyword string.
    pub fn keyword(&self) -> &'static str {
        match self {
            Self::From { .. } => "FROM",
            Self::Run { .. } => "RUN",
            Self::Copy { .. } => "COPY",
            Self::Add { .. } => "ADD",
            Self::Cmd { .. } => "CMD",
            Self::Entrypoint { .. } => "ENTRYPOINT",
            Self::Env { .. } => "ENV",
            Self::Arg { .. } => "ARG",
            Self::Expose { .. } => "EXPOSE",
            Self::Label { .. } => "LABEL",
            Self::User { .. } => "USER",
            Self::Workdir { .. } => "WORKDIR",
            Self::Healthcheck { .. } => "HEALTHCHECK",
            Self::Stopsignal { .. } => "STOPSIGNAL",
            Self::Shell { .. } => "SHELL",
            Self::Volume { .. } => "VOLUME",
            Self::Onbuild { .. } => "ONBUILD",
            Self::Raw { .. } => "RAW",
        }
    }
}

impl fmt::Display for InstructionKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.keyword())
    }
}

/// Parsed Dockerfile instruction node with metadata, line number, and span.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DockerfileInstruction {
    /// Zero-based index of instruction in Dockerfile AST.
    pub index: usize,
    /// Line number in source text (1-indexed).
    pub line: usize,
    /// Detailed instruction variant and parsed parameters.
    pub kind: InstructionKind,
    /// Raw original line text from source file.
    pub raw: String,
    /// Span location in source code.
    pub span: Span,
}
