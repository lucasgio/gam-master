#!/bin/bash
set -e

# Define variables
REPO="giolabs/gam"
BINARY_NAME="gam"
INSTALL_DIR="/usr/local/bin"

# Detect OS and Arch
OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
    Linux)
        ASSET_OS="linux"
        ;;
    Darwin)
        ASSET_OS="macos"
        ;;
    *)
        echo "Unsupported OS: $OS"
        exit 1
        ;;
esac

case "$ARCH" in
    x86_64)
        ASSET_ARCH="amd64"
        ;;
    arm64|aarch64)
        if [ "$OS" = "Darwin" ]; then
            ASSET_ARCH="arm64"
        else
            echo "Unsupported Architecture on Linux: $ARCH"
            exit 1
        fi
        ;;
    *)
        echo "Unsupported Architecture: $ARCH"
        exit 1
        ;;
esac

ASSET_NAME="gam-${ASSET_OS}-${ASSET_ARCH}.tar.gz"
DOWNLOAD_URL="https://github.com/${REPO}/releases/latest/download/${ASSET_NAME}"

echo "Detected ${OS} ${ARCH}"
echo "Downloading ${ASSET_NAME} from ${DOWNLOAD_URL}..."

# Create a temporary directory
TMP_DIR=$(mktemp -d)
trap 'rm -rf "$TMP_DIR"' EXIT

# Download and extract
curl -fsSL "$DOWNLOAD_URL" -o "$TMP_DIR/$ASSET_NAME"
tar -xzf "$TMP_DIR/$ASSET_NAME" -C "$TMP_DIR"

# Install
echo "Installing to ${INSTALL_DIR} (requires sudo)..."
sudo mv "$TMP_DIR/$BINARY_NAME" "$INSTALL_DIR/"
sudo chmod +x "$INSTALL_DIR/$BINARY_NAME"

echo "✅ gam installed successfully to ${INSTALL_DIR}/${BINARY_NAME}"
echo "Try running 'gam --help'"