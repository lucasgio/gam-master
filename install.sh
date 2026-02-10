#!/bin/bash

# Git Account Manager (gam) Installer
# Installs gam by downloading the latest release for your OS/Arch

set -e

OWNER="giolabs"
REPO="gam"
BINARY="gam"
INSTALL_DIR_SYSTEM="/usr/local/bin"
INSTALL_DIR_USER="$HOME/.local/bin"

echo "🔑 Git Account Manager (gam) Installer"
echo "========================================"

# Detect OS
OS="$(uname -s)"
case "$OS" in
    Linux*)     OS_TYPE="linux-gnu" ;;
    Darwin*)    OS_TYPE="apple-darwin" ;;
    *)          echo "❌ Unsupported OS: $OS"; exit 1 ;;
esac

# Detect Architecture
ARCH="$(uname -m)"
case "$ARCH" in
    x86_64)    ARCH_TYPE="x86_64" ;;
    aarch64|arm64)   ARCH_TYPE="aarch64" ;;
    *)         echo "❌ Unsupported architecture: $ARCH"; exit 1 ;;
esac

TARGET="${ARCH_TYPE}-${OS_TYPE}"
echo "🔍 Detected target: $TARGET"

# Fetch latest release info
echo "🔄 Checking for latest release..."
LATEST_RELEASE_URL="https://api.github.com/repos/$OWNER/$REPO/releases/latest"
RELEASE_JSON=$(curl -s "$LATEST_RELEASE_URL")

# Check if we got a valid response
if echo "$RELEASE_JSON" | grep -q "Not Found"; then
    echo "❌ Could not find latest release for $OWNER/$REPO."
    echo "   Ensure the repository is public and has releases."
    exit 1
fi

TAG_NAME=$(echo "$RELEASE_JSON" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')
if [ -z "$TAG_NAME" ]; then
    echo "❌ Could not determine latest version."
    exit 1
fi

echo "📦 Latest version: $TAG_NAME"

# Construct download URL (matching release.yml naming convention)
# Naming format from release.yml: gam-{version}-{target}.tar.gz
# VERSION should be stripped of 'v' prefix usually, but `release.yml` uses:
# PKG_BASENAME="${BIN_NAME}-${VERSION}-${{ matrix.target }}"
# where VERSION="${GITHUB_REF#refs/tags/}" which includes 'v' if tag has it?
# Actually release.yml says: VERSION="${GITHUB_REF#refs/tags/}"
# If tag is v0.1.0, VERSION is v0.1.0. 
# So PKG_BASENAME is gam-v0.1.0-x86_64-apple-darwin
# And archive is .tar.gz (for non-windows)

# Strip 'v' from tag if needed? 
# release.yml just takes the tag name directly.
VERSION="$TAG_NAME"
ASSET_NAME="${BINARY}-${VERSION}-${TARGET}.tar.gz"
DOWNLOAD_URL="https://github.com/$OWNER/$REPO/releases/download/$TAG_NAME/$ASSET_NAME"

echo "⬇️  Downloading $ASSET_NAME..."
TMP_DIR=$(mktemp -d)
trap 'rm -rf "$TMP_DIR"' EXIT

if ! curl -fsSL "$DOWNLOAD_URL" -o "$TMP_DIR/$ASSET_NAME"; then
    echo "❌ Download failed."
    echo "   URL tried: $DOWNLOAD_URL"
    exit 1
fi

# Extract
echo "📦 Extracting..."
tar -xzf "$TMP_DIR/$ASSET_NAME" -C "$TMP_DIR"
# The archive contains a folder with the same name as basename
EXTRACTED_DIR="${BINARY}-${VERSION}-${TARGET}"
BINARY_PATH="$TMP_DIR/$EXTRACTED_DIR/$BINARY"

if [ ! -f "$BINARY_PATH" ]; then
    # Fallback: maybe it flattened?
    BINARY_PATH="$TMP_DIR/$BINARY"
fi

if [ ! -f "$BINARY_PATH" ]; then
    echo "❌ Could not find binary in archive."
    exit 1
fi

chmod +x "$BINARY_PATH"

# Install
echo "🚀 Installing..."
if [ -w "$INSTALL_DIR_SYSTEM" ]; then
    cp "$BINARY_PATH" "$INSTALL_DIR_SYSTEM/$BINARY"
    echo "✅ Installed to $INSTALL_DIR_SYSTEM/$BINARY"
elif command -v sudo &> /dev/null; then
    echo "REQUESTING SUDO to install to $INSTALL_DIR_SYSTEM"
    sudo cp "$BINARY_PATH" "$INSTALL_DIR_SYSTEM/$BINARY"
    echo "✅ Installed to $INSTALL_DIR_SYSTEM/$BINARY"
else
    mkdir -p "$INSTALL_DIR_USER"
    cp "$BINARY_PATH" "$INSTALL_DIR_USER/$BINARY"
    echo "✅ Installed to $INSTALL_DIR_USER/$BINARY"
    echo "⚠️  Ensure $INSTALL_DIR_USER is in your PATH."
fi

echo ""
echo "🎉 Installation complete! Run 'gam --help' to get started."