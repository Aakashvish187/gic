# GIC Architecture Overview

GIC is built as a modular Cargo workspace divided into specialized crates:

```
gic/
├── crates/
│   ├── gic-cli/         # Entrypoint CLI, arg parsing, TUI main event loop
│   ├── gic-core/        # Engine state, buffer management, language detectors, starter wizard templates, updater
│   ├── gic-tui/         # Ratatui rendering engine, syntax highlights, status bar, text renderer
│   ├── gic-config/      # Configuration parser
│   └── gic-logging/     # Structured tracing and logger
├── .github/
│   └── workflows/       # CI/CD (ci.yml, release.yml, nightly.yml)
├── examples/            # IaC sample configurations
├── docs/                # Comprehensive documentation
├── packaging/           # DEB & RPM package metadata
└── install.sh           # One-line Linux installer
```
