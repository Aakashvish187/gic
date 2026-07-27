# GIC Architecture & Subsystem Specification

## Overview

GIC (General Infrastructure Console) is built on **Clean Architecture** principles and SOLID software design. The codebase is organized as a Rust workspace to decouple domain logic from external frameworks, terminal rendering, configuration formats, and system calls.

---

## Workspace Layout & Dependency Inversion

```
+-------------------------------------------------------------+
|                          gic-cli                            |
|             (Application Entrypoint & Flag Parsing)         |
+------------------------------+------------------------------+
                               |
       +-----------------------+-----------------------+
       |                                               |
       v                                               v
+---------------+                            +-------------------+
|    gic-tui    |                            |    gic-config     |
| (TUI/Ratatui) |                            | (TOML Config)     |
+-------+-------+                            +---------+---------+
        |                                              |
        +-----------------------+----------------------+
                                |
                                v
                   +-------------------------+
                   |       gic-logging       |
                   |   (Tracing Subscriber)  |
                   +------------+------------+
                                |
                                v
                   +-------------------------+
                   |        gic-core         |
                   |   (Pure Domain Layer)   |
                   +-------------------------+
```

### Crate Roles

1. **`gic-core`**: Core domain logic, text buffer representation, AST parsers, diagnostic rules, intelligence engines (YAML, K8s, Docker, Terraform, Bash, Git), security scanner, and metadata. Standard library dependencies with zero external framework locks.
2. **`gic-config`**: Infrastructure crate loading, validating, and managing TOML application configuration (`AppConfig`, `UIConfig`, `LogConfig`).
3. **`gic-logging`**: Structured logging subsystem wrapping `tracing` and `tracing-subscriber` with panic protection.
4. **`gic-tui`**: Terminal user interface engine using `ratatui` and `crossterm`. Manages viewport positioning, syntax styling, status bar widgets, line numbering, and layout rendering.
5. **`gic-cli`**: Binary bootstrap crate handling CLI flags (`--about`, `--version`, `--config`) and orchestrating shutdown signals.

---

## Milestone Integration Summary (Milestones 0–17)

### Core Editor
- **Milestone 0 & 1 (Bootstrap & Terminal Engine)**: Raw terminal mode guard, alternate screen buffers, mouse event capture, and signal handling.
- **Milestone 2 & 3 (File System Layer)**: UTF-8 safe document loader, large file streaming, atomic saving, and recent file tracker.
- **Milestone 4 (Text Buffer)**: Thread-safe, piece-table text buffer with undo/redo stack, multi-cursor support, and selection range operations.
- **Milestone 5 (Rendering Engine)**: FPS-independent layout pipeline, status bar, line numbers, cursor shape mapping, and theme engine.
- **Milestone 6 (Search & Replace Engine)**: Boyer-Moore-Horspool and regex pattern matching, match navigation, search history, and multi-file replacements.

### Core Framework
- **Milestone 7 (Universal Parsing Engine)**: Tree-sitter AST integration, fallback tokenizer, syntax node extraction, and token stream caching.
- **Milestone 8 (Universal Diagnostics Engine)**: Severity levels (`Error`, `Warning`, `Info`, `Hint`), positional range mapping, quick-fix suggestions, and diagnostic formatters (`Json`, `PlainText`, `PrettyTerminal`).
- **Milestone 9 (Universal Rule Engine)**: Extensible rule registry, validator interfaces, dynamic evaluation context, and execution metrics.

### Infrastructure Intelligence
- **Milestone 10 (YAML Intelligence)**: YAML AST validation, indentation checking, structural linting, and formatting.
- **Milestone 11 (Kubernetes Intelligence)**: K8s resource manifest detection (Pods, Deployments, Services, ConfigMaps), schema validation, and security posture checks.
- **Milestone 12 (Docker Intelligence)**: Dockerfile AST parser, instruction validator, base image check, root user detection, and multi-stage optimization hints.
- **Milestone 13 (Terraform Intelligence)**: HCL block parser, resource/variable/output declarations, provider validation, and security recommendations.
- **Milestone 14 (Linux & Bash Intelligence)**: Shebang detection, shell syntax linting, dangerous command detection (`rm -rf`), and script formatting.
- **Milestone 15 (Git Awareness Engine)**: Repository discovery, status calculation, diff hunk parser, commit history, blame provider, and `.gitignore` matcher.
- **Milestone 16 (Networking Intelligence)**: Port status checking, endpoint diagnostics, and URL/protocol validation.

### Security
- **Milestone 17 (DevSecOps & Security Engine)**: Secret detection patterns (AWS keys, GitHub tokens, RSA private keys), vulnerability analysis, security severity categorization, and remediation quick-fixes.

---

## Thread Safety & Memory Model

- **Safe Rust Guarantee**: 100% safe Rust codebase with 0 `unsafe` blocks.
- **Concurrency**: Immutable state passing where possible. Parallel processing in rule evaluation and security scanning utilizes `rayon` thread pools and `dashmap` for lock-free read access.
