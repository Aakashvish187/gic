#!/usr/bin/env bash
set -e

# GIC One-Line Installer Script
# Usage: curl -fsSL https://raw.githubusercontent.com/Aakashvish187/gic/main/install.sh | bash

REPO="Aakashvish187/gic"
BINARY_NAME="gic"
INSTALL_DIR="/usr/local/bin"

echo "=================================================="
echo "          Installing GIC Terminal Editor          "
echo "=================================================="

# Detect OS Architecture
ARCH=$(uname -m)
case "$ARCH" in
  x86_64)
    ASSET_ARCH="x86_64"
    ;;
  aarch64|arm64)
    ASSET_ARCH="aarch64"
    ;;
  *)
    echo "Unsupported architecture: $ARCH"
    exit 1
    ;;
esac

# Fetch latest release download URL
echo "--> Fetching latest release information..."
LATEST_RELEASE_URL="https://api.github.com/repos/$REPO/releases/latest"
DOWNLOAD_URL=$(curl -s "$LATEST_RELEASE_URL" | grep "browser_download_url" | grep "$ASSET_ARCH" | cut -d '"' -f 4 | head -n 1)

if [ -z "$DOWNLOAD_URL" ]; then
  echo "Error: Could not find release package for architecture $ASSET_ARCH."
  echo "Fallback: Building from source if Cargo is installed..."
  if command -v cargo &> /dev/null; then
    cargo install --git "https://github.com/$REPO.git" gic-cli --force
    echo "GIC installed successfully via Cargo!"
    exit 0
  else
    echo "Error: Cargo is not installed. Failed to install GIC."
    exit 1
  fi
fi

# Detect Distribution type
if [ -f /etc/debian_version ]; then
    DISTRO_TYPE="debian"
elif [ -f /etc/redhat-release ]; then
    DISTRO_TYPE="rhel"
else
    DISTRO_TYPE="other"
fi

if [ "$DISTRO_TYPE" = "debian" ]; then
    echo "--> Detected Debian/Ubuntu based system. Fetching .deb package..."
    DEB_URL=$(curl -s "$LATEST_RELEASE_URL" | grep "browser_download_url" | grep "\.deb" | grep "$ASSET_ARCH" | cut -d '"' -f 4 | head -n 1)
    if [ -n "$DEB_URL" ]; then
        TMP_DEB=$(mktemp -d)/gic.deb
        curl -fsSL "$DEB_URL" -o "$TMP_DEB"
        echo "--> Installing via dpkg..."
        sudo dpkg -i "$TMP_DEB"
        rm -f "$TMP_DEB"
        INSTALLED_VIA_PKG=1
    fi
elif [ "$DISTRO_TYPE" = "rhel" ]; then
    echo "--> Detected RHEL/Fedora based system. Fetching .rpm package..."
    RPM_URL=$(curl -s "$LATEST_RELEASE_URL" | grep "browser_download_url" | grep "\.rpm" | grep "$ASSET_ARCH" | cut -d '"' -f 4 | head -n 1)
    if [ -n "$RPM_URL" ]; then
        TMP_RPM=$(mktemp -d)/gic.rpm
        curl -fsSL "$RPM_URL" -o "$TMP_RPM"
        echo "--> Installing via rpm..."
        sudo rpm -Uvh "$TMP_RPM"
        rm -f "$TMP_RPM"
        INSTALLED_VIA_PKG=1
    fi
fi

if [ -z "$INSTALLED_VIA_PKG" ]; then
    echo "--> Downloading GIC archive (tar.gz) from $DOWNLOAD_URL..."
    TMP_DIR=$(mktemp -d)
    curl -fsSL "$DOWNLOAD_URL" -o "$TMP_DIR/gic.tar.gz"

    echo "--> Extracting archive..."
    tar -xzf "$TMP_DIR/gic.tar.gz" -C "$TMP_DIR"

    # Check permissions for /usr/local/bin
    if [ -w "$INSTALL_DIR" ]; then
      mv "$TMP_DIR/gic" "$INSTALL_DIR/$BINARY_NAME"
      chmod +x "$INSTALL_DIR/$BINARY_NAME"
    else
      echo "--> System permissions required to write to $INSTALL_DIR..."
      sudo mv "$TMP_DIR/gic" "$INSTALL_DIR/$BINARY_NAME"
      sudo chmod +x "$INSTALL_DIR/$BINARY_NAME"
    fi
    rm -rf "$TMP_DIR"
fi

echo "=================================================="
echo "      GIC installation complete! Version:"
$BINARY_NAME --version || echo "gic installed to $INSTALL_DIR/gic"
echo "=================================================="
