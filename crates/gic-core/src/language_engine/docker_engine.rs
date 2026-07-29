//! # Dockerfile Language Engine

use super::{Completion, CompletionKind, EngineDiagnostic, EngineQuickFix, HoverInfo, LanguageEngine};

pub struct DockerEngine;

impl DockerEngine {
    pub fn new() -> Self {
        Self
    }

    fn check_dockerfile(&self, content: &str) -> Vec<EngineDiagnostic> {
        let mut diagnostics = Vec::new();
        let lines: Vec<&str> = content.lines().collect();
        let mut has_from = false;
        let mut last_from_row = 0;

        for (row, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            let upper = trimmed.to_uppercase();

            // Check FROM
            if upper.starts_with("FROM ") {
                has_from = true;
                last_from_row = row;
                let image = trimmed[5..].trim();
                // Check for latest tag
                if image.ends_with(":latest") {
                    let col = line.find(image).unwrap_or(5);
                    diagnostics.push(
                        EngineDiagnostic::warning(row, col,
                            "Using ':latest' tag. Pin to a specific version for reproducible builds.",
                            "docker")
                            .with_code("DKR001")
                            .with_length(image.len())
                    );
                }
                // Check for missing tag
                if !image.contains(':') && !image.contains(" AS ") && !image.contains(" as ") && image != "scratch" {
                    let col = line.find(image).unwrap_or(5);
                    diagnostics.push(
                        EngineDiagnostic::warning(row, col,
                            "Image has no tag. This defaults to ':latest'. Pin to a specific version.",
                            "docker")
                            .with_code("DKR001")
                            .with_length(image.len())
                    );
                }
            }

            // Check RUN with apt-get without -y
            if upper.starts_with("RUN ") {
                let cmd = &trimmed[4..];
                if cmd.contains("apt-get install") && !cmd.contains("-y") && !cmd.contains("--yes") {
                    diagnostics.push(
                        EngineDiagnostic::warning(row, 4,
                            "apt-get install without '-y' flag will fail in non-interactive mode",
                            "docker")
                            .with_code("DKR002")
                    );
                }
                // Check for apt-get without no-install-recommends
                if cmd.contains("apt-get install") && !cmd.contains("--no-install-recommends") {
                    diagnostics.push(
                        EngineDiagnostic::hint(row, 4,
                            "Consider using '--no-install-recommends' to reduce image size",
                            "docker")
                            .with_code("DKR003")
                    );
                }
                // Check for using sudo
                if cmd.trim_start().starts_with("sudo ") {
                    diagnostics.push(
                        EngineDiagnostic::warning(row, 4,
                            "Avoid using 'sudo' in Dockerfiles. Commands run as root by default.",
                            "docker")
                            .with_code("DKR004")
                    );
                }
            }

            // Check COPY vs ADD
            if upper.starts_with("ADD ") && !trimmed.contains("http://") && !trimmed.contains("https://") && !trimmed.contains(".tar") {
                diagnostics.push(
                    EngineDiagnostic::hint(row, 0,
                        "Use COPY instead of ADD for simple file copying. ADD has extra features (URLs, tar extraction) that may cause unexpected behavior.",
                        "docker")
                        .with_code("DKR005")
                );
            }

            // Check for EXPOSE without protocol
            if upper.starts_with("EXPOSE ") {
                let port_str = trimmed[7..].trim();
                if !port_str.contains('/') && !port_str.is_empty() {
                    diagnostics.push(
                        EngineDiagnostic::hint(row, 7,
                            format!("Consider specifying protocol: 'EXPOSE {}/tcp'", port_str),
                            "docker")
                            .with_code("DKR006")
                    );
                }
            }

            // Check for unknown instructions
            let known_instructions = [
                "FROM", "RUN", "CMD", "LABEL", "EXPOSE", "ENV", "ADD", "COPY",
                "ENTRYPOINT", "VOLUME", "USER", "WORKDIR", "ARG", "ONBUILD",
                "STOPSIGNAL", "HEALTHCHECK", "SHELL",
            ];
            let first_word = trimmed.split_whitespace().next().unwrap_or("");
            if !first_word.is_empty()
                && first_word.chars().all(|c| c.is_uppercase() || c == '_')
                && !known_instructions.contains(&first_word)
            {
                diagnostics.push(
                    EngineDiagnostic::error(row, 0,
                        format!("Unknown instruction: '{}'", first_word),
                        "docker")
                        .with_code("DKR007")
                        .with_length(first_word.len())
                );
            }
        }

        if !has_from && !lines.is_empty() {
            diagnostics.push(
                EngineDiagnostic::error(0, 0, "Dockerfile must start with a FROM instruction", "docker")
                    .with_code("DKR008")
            );
        }

        diagnostics
    }
}

impl LanguageEngine for DockerEngine {
    fn name(&self) -> &'static str { "Dockerfile" }
    fn id(&self) -> &'static str { "docker" }

    fn diagnostics(&self, content: &str) -> Vec<EngineDiagnostic> {
        self.check_dockerfile(content)
    }

    fn completions(&self, _content: &str, row: usize, _col: usize) -> Vec<Completion> {
        let mut completions = Vec::new();

        let instructions = [
            ("FROM", "Base image", "FROM "),
            ("RUN", "Execute command", "RUN "),
            ("COPY", "Copy files", "COPY . ."),
            ("CMD", "Default command", "CMD [\"executable\"]"),
            ("EXPOSE", "Expose port", "EXPOSE 8080"),
            ("ENV", "Set environment variable", "ENV KEY=value"),
            ("WORKDIR", "Set working directory", "WORKDIR /app"),
            ("ENTRYPOINT", "Container entrypoint", "ENTRYPOINT [\"executable\"]"),
            ("ARG", "Build argument", "ARG VERSION=latest"),
            ("VOLUME", "Create mount point", "VOLUME /data"),
            ("USER", "Set user", "USER appuser"),
            ("LABEL", "Add metadata", "LABEL maintainer=\"name\""),
            ("HEALTHCHECK", "Health check", "HEALTHCHECK CMD curl -f http://localhost/ || exit 1"),
        ];

        for (name, detail, insert) in &instructions {
            completions.push(Completion::new(*name, *insert, CompletionKind::Keyword).with_detail(*detail));
        }

        completions
    }

    fn hover(&self, content: &str, row: usize, _col: usize) -> Option<HoverInfo> {
        let line = content.lines().nth(row)?;
        let instruction = line.trim().split_whitespace().next()?.to_uppercase();

        match instruction.as_str() {
            "FROM" => Some(HoverInfo::new("FROM", "Sets the base image for subsequent instructions.")
                .with_syntax("FROM image:tag\nFROM image:tag AS stage")
                .with_best_practice("Use specific image tags. Use multi-stage builds to reduce final image size.")),
            "RUN" => Some(HoverInfo::new("RUN", "Executes a command and commits the result as a new layer.")
                .with_syntax("RUN apt-get update && apt-get install -y package")
                .with_best_practice("Chain commands with && to reduce layers. Clean up in the same RUN.")),
            "COPY" => Some(HoverInfo::new("COPY", "Copies files from the build context into the container.")
                .with_syntax("COPY [--chown=user:group] <src> <dest>")
                .with_best_practice("Use .dockerignore to exclude unnecessary files.")),
            "CMD" => Some(HoverInfo::new("CMD", "Default command to run when the container starts. Only the last CMD takes effect.")
                .with_syntax("CMD [\"executable\", \"param1\"]\nCMD command param1")),
            "EXPOSE" => Some(HoverInfo::new("EXPOSE", "Documents which ports the container listens on. Does not actually publish the port.")
                .with_syntax("EXPOSE 8080/tcp")),
            "ENV" => Some(HoverInfo::new("ENV", "Sets an environment variable available during build and at runtime.")
                .with_syntax("ENV MY_VAR=value")),
            "WORKDIR" => Some(HoverInfo::new("WORKDIR", "Sets the working directory for subsequent instructions.")
                .with_syntax("WORKDIR /app")
                .with_best_practice("Use absolute paths. Create directories with mkdir before WORKDIR.")),
            "HEALTHCHECK" => Some(HoverInfo::new("HEALTHCHECK", "Tells Docker how to test if the container is still working.")
                .with_syntax("HEALTHCHECK --interval=30s CMD curl -f http://localhost/ || exit 1")
                .with_best_practice("Always add a health check for production containers.")),
            _ => None,
        }
    }

    fn smart_enter(&self, line: &str) -> Option<String> {
        let trimmed = line.trim();
        if trimmed.starts_with("FROM") {
            Some("WORKDIR /app".to_string())
        } else if trimmed.starts_with("WORKDIR") {
            Some("COPY . .".to_string())
        } else {
            None
        }
    }

    fn template_expansion(&self, keyword: &str) -> Option<String> {
        match keyword.to_lowercase().as_str() {
            "node" => Some("FROM node:20-alpine AS builder\nWORKDIR /app\nCOPY package*.json ./\nRUN npm ci\nCOPY . .\nRUN npm run build\n\nFROM node:20-alpine\nWORKDIR /app\nCOPY --from=builder /app/dist ./dist\nCOPY package*.json ./\nRUN npm ci --production\nEXPOSE 3000\nCMD [\"npm\", \"start\"]".to_string()),
            "python" => Some("FROM python:3.11-slim\nWORKDIR /app\nCOPY requirements.txt .\nRUN pip install --no-cache-dir -r requirements.txt\nCOPY . .\nEXPOSE 8000\nCMD [\"python\", \"app.py\"]".to_string()),
            "rust" => Some("FROM rust:1.75-slim AS builder\nWORKDIR /usr/src/app\nCOPY . .\nRUN cargo build --release\n\nFROM debian:bookworm-slim\nWORKDIR /app\nCOPY --from=builder /usr/src/app/target/release/█ .\nEXPOSE 8080\nCMD [\"./█\"]".to_string()),
            _ => None,
        }
    }
}
