# Changelog

All notable changes to GIC (General Infrastructure Console) are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [0.1.0] - 2026-07-27

### Added - Milestones 0–17 Integration & Production Release

#### Core Editor Subsystems
- **Bootstrap & Terminal Engine**: Added Crossterm and Ratatui terminal abstraction with RAII panic safety (`TerminalGuard`), FPS-independent frame scheduling, and mouse event capture.
- **File System Layer**: Integrated streaming file reader/writer, atomic saving, UTF-8 safety checks, and recent file tracking.
- **Text Buffer**: Added thread-safe piece-table buffer supporting multi-cursor editing, selections, undo/redo history stack, and line manipulation.
- **Rendering Engine**: Implemented pipeline layout computation, status bar widget, gutter line numbering (absolute & relative), cursor shape rendering, and theme system (`GicDark`, `GicLight`, `HighContrast`).
- **Search & Replace Engine**: Built Boyer-Moore-Horspool matcher and regex search with replacement preview, search history, and multi-file search capabilities.

#### Core Framework
- **Universal Parsing Engine**: Integrated Tree-sitter AST parsers with fallback tokenizer, syntax token stream caching, and incremental parsing support.
- **Universal Diagnostics Engine**: Implemented diagnostic engine with severity classification (`Error`, `Warning`, `Info`, `Hint`), positional range mapping, quick-fix edits, and output formatters (`Json`, `PlainText`, `PrettyTerminal`).
- **Universal Rule Engine**: Created extensible validator registry, rule evaluation context, custom validation metrics, and configurable rules.

#### Infrastructure Intelligence
- **YAML Intelligence**: Added YAML AST parsing, indentation validator, structural error detection, and formatting engine.
- **Kubernetes Intelligence**: Integrated manifest detector, API version validator, resource kind inspector (Pod, Deployment, Service, ConfigMap), and security posture analyzer.
- **Docker Intelligence**: Implemented Dockerfile parser, multi-stage build validator, base image checker, and root user warning rules.
- **Terraform Intelligence**: Added HCL block parser, resource/variable/output declarations, provider validator, and security rule engine.
- **Linux & Bash Intelligence**: Implemented Shebang detector, dangerous command identifier (`rm -rf /`), syntax validator, and script formatter.
- **Git Awareness Engine**: Built repository discovery, status tracker, diff hunk calculator, commit history inspector, blame provider, and `.gitignore` evaluator.
- **Networking Intelligence**: Implemented network port status validator, host endpoint diagnostic provider, and protocol checker.

#### Security & Integration
- **DevSecOps Engine**: Added secret scanner for AWS keys, RSA private keys, GitHub tokens, and sensitive strings; integrated security rule suggestions and remediation hints.
- **Centralized Project Metadata**: Created `gic_core::metadata` module with `ProjectMetadata` constants, `AboutInfo` bundle, and `DefaultAboutProvider` exposing `--about` and `--version` CLI flags.
- **Documentation Suite**: Standardized repository documentation with `README.md`, `ARCHITECTURE.md`, `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md`, `SECURITY.md`, `CHANGELOG.md`, `AUTHORS.md`, `LICENSE`, `ROADMAP.md`, `INSTALL.md`, and `BUILD.md`.
