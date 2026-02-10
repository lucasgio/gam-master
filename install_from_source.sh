#!/bin/bash

# Git Manager Command (gmc) Installation Script
# This script compiles and installs the Git Manager Command tool

set -e

echo "🔑 Git Manager Command (gmc) Installation"
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

echo "🔄 Compiling Git Manager Command (gmc)..."
cargo build --release

if [ $? -eq 0 ]; then
    echo "✅ Compilation successful!"
else
    echo "❌ Compilation failed!"
    exit 1
fi

# Ask user if they want to install globally
echo ""
read -p "Do you want to install Git Manager Command (gmc) globally? (requires sudo) [y/N]: " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    echo "🔄 Installing globally..."
    sudo cp target/release/gmc /usr/local/bin/
    echo "✅ gmc installed to /usr/local/bin/gmc"
    echo ""
    echo "You can now run: gmc"
else
    echo "ℹ️  You can run Git Manager Command using:"
    echo "   cd $(pwd)"
    echo "   ./target/release/gmc"
fi

echo ""
echo "🎉 Installation complete!"
echo ""
echo "📖 Usage:"
echo "   gmc          # Interactive mode"
echo "   gmc add      # Add new account"
echo "   gmc list     # List accounts"
echo "   gmc switch   # Switch accounts"
echo "   gmc status   # Show current account"
echo ""
echo "📚 See README.md for detailed documentation"