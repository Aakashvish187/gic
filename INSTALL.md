# Installation Guide for GIC

This guide provides instructions for installing and running **GIC (General Infrastructure Console)** on Linux, macOS, and Windows systems.

---

## Prerequisites

- **Rust Toolchain**: Rust 1.75.0 or higher is required.
  Install Rust via `rustup`:
  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```

---

## Installation Methods

### Building from Source

1. **Clone the Repository**:
   ```bash
   git clone https://github.com/Aakashvish187/gic.git
   cd gic
   ```

2. **Build and Install Release Binary**:
   ```bash
   cargo install --path crates/gic-cli
   ```

3. **Verify Installation**:
   ```bash
   gic --about
   ```

---

## Operating System Specific Notes

### Linux / WSL
Ensure terminal color support and `git` binaries are installed:
```bash
sudo apt update && sudo apt install -y build-essential git
```

### macOS
Ensure Xcode Command Line Tools are installed:
```bash
xcode-select --install
```

### Windows (PowerShell)
GIC works out of the box in Windows Terminal or PowerShell.
Ensure Rust binaries (`~/.cargo/bin`) are present on your `%PATH%`.
