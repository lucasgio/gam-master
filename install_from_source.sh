#!/bin/bash

# Git Account Manager (gam) Installation Script
# This script compiles and installs the Git Account Manager tool

set -e

echo "🔑 Git Account Manager (gam) Installation"
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

echo "🔄 Compiling Git Account Manager (gam)..."
cargo build --release

if [ $? -eq 0 ]; then
    echo "✅ Compilation successful!"
else
    echo "❌ Compilation failed!"
    exit 1
fi

# Ask user if they want to install globally
echo ""
read -p "Do you want to install Git Account Manager (gam) globally? (requires sudo) [y/N]: " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    echo "🔄 Installing globally..."
    sudo cp target/release/gam /usr/local/bin/
    echo "✅ gam installed to /usr/local/bin/gam"
    echo ""
    echo "You can now run: gam"
else
    echo "ℹ️  You can run Git Account Manager using:"
    echo "   cd $(pwd)"
    echo "   ./target/release/gam"
fi

echo ""
echo "🎉 Installation complete!"
echo ""
echo "📖 Usage:"
echo "   gam          # Interactive mode"
echo "   gam add      # Add new account"
echo "   gam list     # List accounts"
echo "   gam switch   # Switch accounts"
echo "   gam status   # Show current account"
echo ""
echo "📚 See README.md for detailed documentation"