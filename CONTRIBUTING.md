# Contributing to GIC

Thank you for your interest in contributing to **GIC (General Infrastructure Console)**!

We welcome contributions from everyone. Please take a moment to review these guidelines before submitting code or issues.

---

## Code of Conduct

All contributors are expected to adhere to our [Code of Conduct](CODE_OF_CONDUCT.md).

---

## Architectural Principles & Coding Standards

1. **Clean Architecture & SOLID**:
   - Keep domain logic strictly inside `gic-core`. Avoid coupling `gic-core` to terminal or UI frameworks.
   - Separate concerns across crates: `gic-config` for settings, `gic-logging` for telemetry, `gic-tui` for rendering, `gic-cli` for binary initialization.
2. **Zero `unsafe` Policy**:
   - Unsafe code is strictly forbidden. The entire codebase must compile using 100% safe Rust.
3. **Comprehensive Unit Testing**:
   - Every module must include unit tests in a `mod tests` block.
   - All tests must pass: `cargo test --workspace`.
4. **Code Quality & Formatting**:
   - Format all code with `cargo fmt`.
   - Ensure `cargo clippy --workspace -- -D warnings` produces zero warnings.

---

## Contribution Workflow

1. **Fork the Repository**:
   Fork `https://github.com/Aakashvish187/gic` and clone your fork locally.

2. **Create a Feature Branch**:
   ```bash
   git checkout -b feature/amazing-feature
   ```

3. **Implement Changes & Run Verification**:
   ```bash
   cargo fmt --check
   cargo clippy --workspace -- -D warnings
   cargo test --workspace
   ```

4. **Commit & Push**:
   Write clear, imperative commit messages.
   ```bash
   git commit -m "feat(diagnostics): add custom Kubernetes rule validator"
   git push origin feature/amazing-feature
   ```

5. **Open a Pull Request**:
   Target the `main` branch of `Aakashvish187/gic`. Ensure all CI checks pass.
