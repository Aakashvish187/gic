# Build & Development Instructions

This document covers build configurations, target profiles, and verification tools for **GIC (General Infrastructure Console)**.

---

## Build Targets & Profiles

### Development Profile (`dev`)
Optimized for fast incremental compilation and full debug symbols:
```bash
cargo build --workspace
```

### Release Profile (`release`)
Optimized for maximum runtime performance, Link-Time Optimization (LTO), and compact binary size:
```bash
cargo build --release --workspace
```
*Key Cargo release settings configured in `Cargo.toml`:*
- `opt-level = 3`
- `lto = true`
- `codegen-units = 1`
- `panic = "unwind"`
- `strip = true`

---

## Running Quality Checks

### Automated Test Suite
Run unit tests across all 5 workspace crates (`gic-core`, `gic-config`, `gic-logging`, `gic-tui`, `gic-cli`):
```bash
cargo test --workspace
```

### Formatting Check
```bash
cargo fmt --check
```

### Clippy Linter Check
Enforce strict zero-warning policy:
```bash
cargo clippy --workspace -- -D warnings
```

---

## Running Specific Packages

```bash
# Run GIC CLI
cargo run -p gic-cli

# Run GIC CLI with About information
cargo run -p gic-cli -- --about

# Run gic-core tests only
cargo test -p gic-core
```
