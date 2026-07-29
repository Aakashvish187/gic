# GIC Installation Guide

This document outlines all supported installation and upgrade methods for GIC.

## 1. Automatic One-Line Script (Recommended)

Run the following command in your terminal:
```bash
curl -fsSL https://raw.githubusercontent.com/Aakashvish187/gic/main/install.sh | bash
```

This script automatically:
1. Detects your CPU architecture (`x86_64` vs `aarch64`).
2. Downloads the latest release archive.
3. Installs the `gic` executable to `/usr/local/bin`.

---

## 2. Debian / Ubuntu (`.deb`)

Download the `.deb` package from the [GitHub Releases Page](https://github.com/Aakashvish187/gic/releases):
```bash
sudo dpkg -i gic.deb
```

---

## 3. Fedora / RHEL / CentOS (`.rpm`)

Download the `.rpm` package from the [GitHub Releases Page](https://github.com/Aakashvish187/gic/releases):
```bash
sudo dnf install gic.rpm
```

---

## 4. Upgrading GIC

You can upgrade GIC to the latest release at any time using the built-in update subcommand:
```bash
gic update
```
