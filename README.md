# GIC - Infrastructure-as-Code Terminal Editor & Intelligence Platform

[![CI](https://github.com/Aakashvish187/gic/actions/workflows/ci.yml/badge.svg)](https://github.com/Aakashvish187/gic/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/Aakashvish187/gic?color=blue)](https://github.com/Aakashvish187/gic/releases)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)

**GIC** is a ultra-fast, zero-overhead Terminal User Interface (TUI) text editor and intelligence platform engineered specifically for DevOps engineers and SREs working with Infrastructure-as-Code (IaC).

---

## 🚀 Key Features

- **⚡ Blazing Fast**: Built in Rust for sub-millisecond keypress latency and sub-50ms startup times.
- **🪄 Starter Wizard Engine**: Creates complete, industry-standard boilerplates for **Kubernetes, Docker, Docker Compose, Terraform, Ansible, and GitHub Actions**.
- **🔍 Full-Featured Find (`Ctrl+F`)**: Instant search with realtime visual match highlighting across your document.
- **🛡️ Built-in Schema Validation**: Live diagnostic error and warning checking for IaC files.
- **🔄 Auto-Update Subcommand (`gic update`)**: Self-updating binary powered directly by GitHub Releases.
- **📦 Multi-Target Linux Packaging**: Official support for **DEB**, **RPM**, **tar.gz**, and direct binary downloads.

---

## 📥 Quick Installation

### One-Line Shell Installer (Linux)
```bash
curl -fsSL https://raw.githubusercontent.com/Aakashvish187/gic/main/install.sh | bash
```

### Install via DEB (Debian / Ubuntu)
```bash
sudo dpkg -i gic.deb
```

### Install via RPM (RHEL / Fedora / CentOS)
```bash
sudo dnf install gic.rpm
```

For complete setup instructions, see the [Installation Guide](docs/INSTALLATION.md).

---

## ⚡ Quick Start

Open any existing file or start a new configuration:
```bash
# Open Kubernetes Deployment (launches starter wizard if new file)
gic deployment.yaml

# Open Docker Compose configuration
gic docker-compose.yml

# Open Terraform Infrastructure file
gic main.tf
```

---

## ⌨️ Keybindings

| Keybinding | Mode | Description |
|---|---|---|
| `Ctrl+F` | Normal/Insert | Open search bar & highlight matches |
| `Enter` | Search | Jump to next search match |
| `Esc` | Search/Insert | Return to Normal mode / Clear search |
| `i` | Normal | Enter Insert mode |
| `Ctrl+Z` | Normal | Undo last change |
| `Ctrl+Y` | Normal | Redo change |

See the complete [Keyboard Shortcuts Cheat Sheet](docs/KEYBOARD_SHORTCUTS.md).

---

## 📚 Documentation

- [Installation Guide](docs/INSTALLATION.md)
- [Keyboard Shortcuts](docs/KEYBOARD_SHORTCUTS.md)
- [Architecture Overview](docs/ARCHITECTURE.md)
- [Starter Wizard Guide](docs/STARTER_WIZARD.md)

---

## 📄 License

GIC is dual-licensed under the terms of both the **MIT License** and the **Apache License (Version 2.0)**.