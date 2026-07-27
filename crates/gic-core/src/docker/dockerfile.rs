//! Dockerfile Parser and AST Generator for GIC.
//!
//! Scans raw Dockerfile text, handles line continuations (`\`), strips comments,
//! and parses instructions into a `DockerfileAST`.

use crate::docker::errors::DockerResult;
use crate::docker::instructions::{DockerfileInstruction, InstructionKind};
use crate::yaml::parser::{Position, Span};

/// Complete parsed Dockerfile AST representation.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DockerfileAST {
    /// Ordered list of parsed instructions.
    pub instructions: Vec<DockerfileInstruction>,
    /// Raw original source text.
    pub source: String,
}

impl DockerfileAST {
    /// Returns true if the AST contains no instructions.
    pub fn is_empty(&self) -> bool {
        self.instructions.is_empty()
    }
}

/// High-performance Dockerfile parser.
#[derive(Debug, Clone, Default)]
pub struct DockerfileParser;

impl DockerfileParser {
    /// Creates a new DockerfileParser.
    pub fn new() -> Self {
        Self
    }

    /// Parses raw Dockerfile text into a `DockerfileAST`.
    pub fn parse(&self, source: &str) -> DockerResult<DockerfileAST> {
        let mut instructions = Vec::new();
        let logical_lines = preprocess_logical_lines(source);

        for (idx, (line_num, raw_line)) in logical_lines.into_iter().enumerate() {
            let trimmed = raw_line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            let start_pos = Position::new(line_num, 1, 0);
            let end_pos = Position::new(line_num, raw_line.len().max(1), 0);
            let span = Span::new(start_pos, end_pos);

            let kind = parse_instruction_kind(trimmed)?;

            instructions.push(DockerfileInstruction {
                index: idx,
                line: line_num,
                kind,
                raw: raw_line,
                span,
            });
        }

        Ok(DockerfileAST {
            instructions,
            source: source.to_string(),
        })
    }
}

fn preprocess_logical_lines(source: &str) -> Vec<(usize, String)> {
    let mut logical_lines = Vec::new();
    let mut current_line = String::new();
    let mut start_line_num = 1;

    for (idx, line) in source.lines().enumerate() {
        let line_num = idx + 1;
        let trimmed = line.trim();

        if current_line.is_empty() {
            start_line_num = line_num;
        }

        if trimmed.ends_with('\\') {
            current_line.push_str(trimmed.trim_end_matches('\\').trim());
            current_line.push(' ');
        } else {
            current_line.push_str(line);
            logical_lines.push((start_line_num, current_line.clone()));
            current_line.clear();
        }
    }

    if !current_line.is_empty() {
        logical_lines.push((start_line_num, current_line));
    }

    logical_lines
}

fn parse_instruction_kind(line: &str) -> DockerResult<InstructionKind> {
    let parts: Vec<&str> = line.splitn(2, char::is_whitespace).collect();
    if parts.is_empty() {
        return Ok(InstructionKind::Raw {
            keyword: String::new(),
            payload: String::new(),
        });
    }

    let keyword = parts[0].to_uppercase();
    let payload = parts.get(1).map(|s| s.trim()).unwrap_or("");

    match keyword.as_str() {
        "FROM" => parse_from(payload),
        "RUN" => Ok(parse_cmd_or_run(payload, false)),
        "COPY" => parse_copy_or_add(payload, true),
        "ADD" => parse_copy_or_add(payload, false),
        "CMD" => Ok(parse_cmd_or_run(payload, true)),
        "ENTRYPOINT" => Ok(parse_entrypoint(payload)),
        "ENV" => Ok(parse_env(payload)),
        "ARG" => Ok(parse_arg(payload)),
        "EXPOSE" => Ok(parse_expose(payload)),
        "LABEL" => Ok(parse_label(payload)),
        "USER" => Ok(parse_user(payload)),
        "WORKDIR" => Ok(InstructionKind::Workdir {
            path: payload.to_string(),
        }),
        "HEALTHCHECK" => Ok(parse_healthcheck(payload)),
        "STOPSIGNAL" => Ok(InstructionKind::Stopsignal {
            signal: payload.to_string(),
        }),
        "SHELL" => Ok(parse_shell(payload)),
        "VOLUME" => Ok(parse_volume(payload)),
        "ONBUILD" => Ok(InstructionKind::Onbuild {
            trigger: payload.to_string(),
        }),
        _ => Ok(InstructionKind::Raw {
            keyword,
            payload: payload.to_string(),
        }),
    }
}

fn parse_from(payload: &str) -> DockerResult<InstructionKind> {
    let mut platform = None;
    let mut image = String::new();
    let mut stage_alias = None;

    let parts: Vec<&str> = payload.split_whitespace().collect();
    let mut idx = 0;

    while idx < parts.len() {
        let p = parts[idx];
        if p.starts_with("--platform=") {
            platform = Some(p.trim_start_matches("--platform=").to_string());
        } else if image.is_empty() {
            image = p.to_string();
        } else if p.eq_ignore_ascii_case("AS") && idx + 1 < parts.len() {
            stage_alias = Some(parts[idx + 1].to_string());
            break;
        }
        idx += 1;
    }

    Ok(InstructionKind::From {
        image,
        platform,
        stage_alias,
    })
}

fn parse_cmd_or_run(payload: &str, is_cmd: bool) -> InstructionKind {
    let is_exec = payload.starts_with('[') && payload.ends_with(']');
    let args = if is_exec {
        parse_json_string_array(payload)
    } else {
        vec![payload.to_string()]
    };

    if is_cmd {
        InstructionKind::Cmd {
            arguments: args,
            is_exec_form: is_exec,
        }
    } else {
        InstructionKind::Run {
            command: args,
            is_exec_form: is_exec,
        }
    }
}

fn parse_entrypoint(payload: &str) -> InstructionKind {
    let is_exec = payload.starts_with('[') && payload.ends_with(']');
    let args = if is_exec {
        parse_json_string_array(payload)
    } else {
        vec![payload.to_string()]
    };

    InstructionKind::Entrypoint {
        arguments: args,
        is_exec_form: is_exec,
    }
}

fn parse_copy_or_add(payload: &str, is_copy: bool) -> DockerResult<InstructionKind> {
    let mut from_stage = None;
    let mut chown = None;
    let mut positional = Vec::new();

    for token in payload.split_whitespace() {
        if token.starts_with("--from=") {
            from_stage = Some(token.trim_start_matches("--from=").to_string());
        } else if token.starts_with("--chown=") {
            chown = Some(token.trim_start_matches("--chown=").to_string());
        } else {
            positional.push(token.to_string());
        }
    }

    let destination = positional.pop().unwrap_or_default();
    let sources = positional;

    if is_copy {
        Ok(InstructionKind::Copy {
            sources,
            destination,
            from_stage,
            chown,
        })
    } else {
        Ok(InstructionKind::Add {
            sources,
            destination,
            chown,
        })
    }
}

fn parse_env(payload: &str) -> InstructionKind {
    let mut pairs = Vec::new();
    if payload.contains('=') {
        for token in payload.split_whitespace() {
            if let Some((k, v)) = token.split_once('=') {
                pairs.push((k.to_string(), v.to_string()));
            }
        }
    } else {
        let parts: Vec<&str> = payload.splitn(2, char::is_whitespace).collect();
        if parts.len() == 2 {
            pairs.push((parts[0].to_string(), parts[1].to_string()));
        } else if !payload.is_empty() {
            pairs.push((payload.to_string(), String::new()));
        }
    }
    InstructionKind::Env { pairs }
}

fn parse_arg(payload: &str) -> InstructionKind {
    if let Some((k, v)) = payload.split_once('=') {
        InstructionKind::Arg {
            name: k.trim().to_string(),
            default_value: Some(v.trim().to_string()),
        }
    } else {
        InstructionKind::Arg {
            name: payload.trim().to_string(),
            default_value: None,
        }
    }
}

fn parse_expose(payload: &str) -> InstructionKind {
    let ports = payload.split_whitespace().map(|s| s.to_string()).collect();
    InstructionKind::Expose { ports }
}

fn parse_label(payload: &str) -> InstructionKind {
    let mut pairs = Vec::new();
    for token in payload.split_whitespace() {
        if let Some((k, v)) = token.split_once('=') {
            pairs.push((k.to_string(), v.trim_matches('"').to_string()));
        }
    }
    InstructionKind::Label { pairs }
}

fn parse_user(payload: &str) -> InstructionKind {
    if let Some((u, g)) = payload.split_once(':') {
        InstructionKind::User {
            user: u.to_string(),
            group: Some(g.to_string()),
        }
    } else {
        InstructionKind::User {
            user: payload.to_string(),
            group: None,
        }
    }
}

fn parse_healthcheck(payload: &str) -> InstructionKind {
    if payload.eq_ignore_ascii_case("NONE") {
        return InstructionKind::Healthcheck {
            is_none: true,
            interval: None,
            timeout: None,
            start_period: None,
            retries: None,
            command: Vec::new(),
        };
    }

    let mut interval = None;
    let mut timeout = None;
    let mut start_period = None;
    let mut retries = None;
    let mut command = Vec::new();

    let parts: Vec<&str> = payload.split_whitespace().collect();
    let mut idx = 0;

    while idx < parts.len() {
        let p = parts[idx];
        if p.starts_with("--interval=") {
            interval = Some(p.trim_start_matches("--interval=").to_string());
        } else if p.starts_with("--timeout=") {
            timeout = Some(p.trim_start_matches("--timeout=").to_string());
        } else if p.starts_with("--start-period=") {
            start_period = Some(p.trim_start_matches("--start-period=").to_string());
        } else if p.starts_with("--retries=") {
            retries = p.trim_start_matches("--retries=").parse().ok();
        } else if p == "CMD" {
            command = parts[idx + 1..].iter().map(|s| s.to_string()).collect();
            break;
        }
        idx += 1;
    }

    InstructionKind::Healthcheck {
        is_none: false,
        interval,
        timeout,
        start_period,
        retries,
        command,
    }
}

fn parse_shell(payload: &str) -> InstructionKind {
    InstructionKind::Shell {
        shell: parse_json_string_array(payload),
    }
}

fn parse_volume(payload: &str) -> InstructionKind {
    if payload.starts_with('[') && payload.ends_with(']') {
        InstructionKind::Volume {
            paths: parse_json_string_array(payload),
        }
    } else {
        InstructionKind::Volume {
            paths: payload.split_whitespace().map(|s| s.to_string()).collect(),
        }
    }
}

fn parse_json_string_array(raw: &str) -> Vec<String> {
    let trimmed = raw.trim_matches(|c| c == '[' || c == ']' || c == ' ' || c == '\n');
    trimmed
        .split(',')
        .map(|s| s.trim().trim_matches('"').trim_matches('\'').to_string())
        .filter(|s| !s.is_empty())
        .collect()
}
