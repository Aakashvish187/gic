# GIC Release Guide

This document outlines the packaging, release, and installation infrastructure for the General Infrastructure Console (GIC).

## 🚀 Creating a Release

GIC uses GitHub Actions to fully automate the release process. There are no manual packaging or uploading steps required.

To create a new public release:

1. Update the `version` field in the root `Cargo.toml`.
2. Commit and push the version bump to `main`.
3. Create and push a Git tag starting with `v` (e.g., `v1.0.0`).

```bash
git tag v1.0.0
git push origin v1.0.0
```

Once the tag is pushed, the `.github/workflows/release.yml` GitHub Action will trigger automatically.

## ⚙️ GitHub Actions Pipeline

When a version tag is pushed, the release pipeline performs the following tasks:

1. **Builds the Binaries**: Cross-compiles release-optimized Rust binaries for `x86_64` and `aarch64` architectures.
2. **Generates Packages**: 
   - **Debian (`.deb`)**: Builds standard Debian packages using `dpkg-deb` and the `packaging/debian/control` file.
   - **RPM (`.rpm`)**: Builds RPM packages using `rpmbuild` and the `packaging/rpm/gic.spec` specfile.
   - **Tarball (`.tar.gz`)**: Creates a fallback archive containing the raw binary, `LICENSE`, and `README.md`.
3. **Creates the GitHub Release**: Uploads all artifacts, generates `SHA256SUMS.txt`, and publishes the release on the GitHub Releases page.

> [!NOTE]
> The GitHub Action automatically injects the version from the Git tag (`GITHUB_REF`) into the packaging control files, ensuring that the `.deb` and `.rpm` metadata exactly matches the release tag.

## 📦 Installation Guide for Users

We provide a seamless, one-line installer script that detects the user's OS and architecture, and downloads the correct package from the latest GitHub Release.

```bash
curl -fsSL https://raw.githubusercontent.com/Aakashvish187/gic/main/install.sh | bash
```

### How the Installer Works

1. **Architecture Detection**: Determines if the system is `x86_64` or `aarch64`.
2. **Distribution Detection**: Checks `/etc/debian_version` and `/etc/redhat-release`.
3. **Package Installation**:
   - On **Debian/Ubuntu**, it downloads and installs the `.deb` package via `dpkg`.
   - On **RHEL/Fedora**, it downloads and installs the `.rpm` package via `rpm`.
   - On **Unsupported Distributions**, it falls back to downloading the `.tar.gz`, extracting the binary, and moving it to `/usr/local/bin/gic`.
4. **Source Fallback**: If the pre-compiled binary is unavailable, the script attempts to build GIC from source using `cargo install --git`.

## 🔄 Updating GIC

To update GIC to the latest version, users simply re-run the installer script:

```bash
curl -fsSL https://raw.githubusercontent.com/Aakashvish187/gic/main/install.sh | bash
```

The script will fetch the newest release and `dpkg`/`rpm` will automatically handle upgrading the existing package.
