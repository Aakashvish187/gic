# Developer Guide for GIC

Welcome to the GIC developer documentation! This guide will help you understand the architecture of the General Infrastructure Console and how to contribute effectively.

## 1. Project Architecture

GIC uses a Cargo Workspace architecture composed of multiple crates to maintain high modularity.

- **`gic-core`**: The brain of the editor. Contains syntax highlighting, buffer management, language server integrations, auto-completion, and the project starter engine.
- **`gic-tui`**: The user interface layer built on `ratatui`. It is responsible for rendering the buffer, status bar, diagnostics, and handling user input.
- **`gic-cli`**: The command-line interface. Parses arguments, handles initial bootstrap, runs the project wizard if requested, and launches the TUI.
- **`gic-config`**: Manages configuration loading (`gic.toml`), default settings, and user preferences.
- **`gic-logging`**: Structured logging using `tracing` and `tracing-subscriber`.

## 2. Setting Up Your Environment

To build and run GIC locally:

```bash
# Clone the repository
git clone https://github.com/Aakashvish187/gic.git
cd gic

# Build the project
cargo build

# Run tests
cargo test --all

# Run the application in debug mode
cargo run -- --debug
```

## 3. Adding a New Language Engine

Adding support for a new language (e.g., a new IaC tool) involves interacting with `gic-core/src/syntax`.

1. **Define the Language**: Add the language configuration in the registry.
2. **Implement Syntax Highlighting**: Create a regex-based or tree-sitter based highlighter for the language.
3. **Register Diagnostics**: If there is a linter available, implement the diagnostics provider trait for the language.

## 4. Release Process

The release process is fully automated via GitHub Actions. Maintainers should refer to `docs/RELEASE_CHECKLIST.md` before pushing a `v*` tag.

When a tag like `v1.0.0` is pushed:
- `release.yml` triggers.
- It compiles statically linked binaries for x86_64 and ARM64.
- Generates `.tar.gz`, `.deb`, and `.rpm` packages.
- Creates a GitHub release and attaches the assets along with SHA checksums.
