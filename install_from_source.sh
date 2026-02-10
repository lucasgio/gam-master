#!/bin/bash

# Git Account Manager CLI (gam-cli) Installation Script
# This script compiles and installs the Git Account Manager CLI tool

set -e

echo "🔑 Git Account Manager CLI (gam-cli) Installation"
echo "========================================="

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "❌ Rust is not installed. Please install it from https://rustup.rs/"
    exit 1
fi

# Check if we're on macOS
if [[ "$OSTYPE" != "darwin"* ]]; then
    echo "⚠️  This tool is designed for macOS due to Keychain integration."
    echo "   It might work on other systems but some features may not work properly."
fi

echo "🔄 Compiling Git Account Manager CLI (gam-cli)..."
cargo build --release

if [ $? -eq 0 ]; then
    echo "✅ Compilation successful!"
else
    echo "❌ Compilation failed!"
    exit 1
fi

# Ask user if they want to install globally
echo ""
read -p "Do you want to install Git Account Manager CLI (gam-cli) globally? (requires sudo) [y/N]: " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    echo "🔄 Installing globally..."
    sudo cp target/release/gam-cli /usr/local/bin/
    echo "✅ gam-cli installed to /usr/local/bin/gam-cli"
    echo ""
    echo "You can now run: gam-cli"
else
    echo "ℹ️  You can run Git Account Manager CLI using:"
    echo "   cd $(pwd)"
    echo "   ./target/release/gam-cli"
fi

echo ""
echo "🎉 Installation complete!"
echo ""
echo "📖 Usage:"
echo "   gam-cli          # Interactive mode"
echo "   gam-cli add      # Add new account"
echo "   gam-cli list     # List accounts"
echo "   gam-cli switch   # Switch accounts"
echo "   gam-cli status   # Show current account"
echo ""
echo "📚 See README.md for detailed documentation"