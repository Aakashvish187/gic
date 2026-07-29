# GIC Release Checklist

Before tagging a new release (e.g., `v1.0.0`) and triggering the automated deployment pipeline, maintainers must verify the following items to ensure production readiness.

## 1. Local Verifications
- [ ] Run `cargo test --all` to ensure all unit and integration tests pass.
- [ ] Run `cargo clippy --all-targets --all-features -- -D warnings` to ensure no linting warnings.
- [ ] Run `cargo fmt --all -- --check` to ensure formatting adheres to standards.

## 2. Feature Verification (Manual Smoke Tests)
- [ ] Editor starts successfully with no crashes.
- [ ] Typing is smooth (no lag, verifying 200ms startup latency goal).
- [ ] File saving works correctly on disk.
- [ ] Search & Replace functions operate accurately.
- [ ] Undo/Redo stack applies changes and rolls back smoothly.
- [ ] Project Starter (Wizard) generates correct templates (Kubernetes, Docker, etc.).
- [ ] Language Autocomplete, Diagnostics, Quick Fix, and Hover render accurately on sample files.

## 3. Package & Install Checks
- [ ] Update version number in `Cargo.toml` (workspace package version).
- [ ] Update version number in `packaging/debian/control`.
- [ ] Update version number in `packaging/rpm/gic.spec`.
- [ ] `gic update` runs and safely reports whether updates are available.

## 4. Tagging and Releasing
- [ ] Commit all final version bumps: `git commit -m "chore: bump version to vX.Y.Z"`
- [ ] Create an annotated git tag: `git tag -a vX.Y.Z -m "Release vX.Y.Z"`
- [ ] Push the tag: `git push origin vX.Y.Z`

Once pushed, GitHub Actions will automatically:
1. Build the release binaries.
2. Generate `.tar.gz`, `.deb`, and `.rpm` packages.
3. Publish the GitHub Release and attach the assets.
