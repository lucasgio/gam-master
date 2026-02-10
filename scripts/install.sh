#!/usr/bin/env bash

set -euo pipefail

# gam installer (user-friendly, no sudo by default)

REPO="lucasgio/gam"

detect_target() {
  local os arch target
  os="$(uname -s)"
  arch="$(uname -m)"

  case "$os" in
    Linux)
      case "$arch" in
        x86_64|amd64) target="x86_64-unknown-linux-gnu" ;;
        aarch64|arm64) target="aarch64-unknown-linux-gnu" ;;
        *) echo "Unsupported Linux arch: $arch" >&2; exit 1 ;;
      esac
      ;;
    Darwin)
      case "$arch" in
        x86_64) target="x86_64-apple-darwin" ;;
        arm64) target="aarch64-apple-darwin" ;;
        *) echo "Unsupported macOS arch: $arch" >&2; exit 1 ;;
      esac
      ;;
    *)
      echo "Unsupported OS: $os" >&2; exit 1 ;;
  esac

  echo "$target"
}

have_cmd() { command -v "$1" >/dev/null 2>&1; }

download() {
  local url="$1" out="$2"
  if have_cmd curl; then
    curl -fsSL "$url" -o "$out"
  elif have_cmd wget; then
    wget -qO "$out" "$url"
  else
    echo "Please install curl or wget" >&2; exit 1
  fi
}

main() {
  local target url tmpdir tarball bindest binpath instdir
  target="$(detect_target)"
  url="https://github.com/${REPO}/releases/latest/download/gam-${target}.tar.gz"

  tmpdir="$(mktemp -d)"
  trap 'rm -rf "$tmpdir"' EXIT

  echo "Downloading gam for ${target}..."
  tarball="$tmpdir/gam.tar.gz"
  download "$url" "$tarball"

  echo "Extracting..."
  (cd "$tmpdir" && tar -xzf "$tarball")

  # Find the extracted binary regardless of versioned directory name
  binpath="$(find "$tmpdir" -type f -maxdepth 3 -name gam -perm -u+x | head -n1 || true)"
  if [[ -z "$binpath" ]]; then
    # Fallback: just search for file named gam
    binpath="$(find "$tmpdir" -type f -maxdepth 3 -name gam | head -n1 || true)"
  fi
  if [[ -z "$binpath" ]]; then
    echo "Could not locate extracted 'gam' binary" >&2; exit 1
  fi

  # Choose install dir (no sudo by default)
  instdir="${GAM_INSTALL_DIR:-$HOME/.local/bin}"
  mkdir -p "$instdir"
  bindest="$instdir/gam"

  echo "Installing to $bindest"
  install -m 755 "$binpath" "$bindest" 2>/dev/null || {
    # If install fails (e.g., permission), try copy + chmod
    cp "$binpath" "$bindest"
    chmod 755 "$bindest"
  }

  # PATH hint
  case ":${PATH}:" in
    *:"$instdir":*) : ;;
    *)
      echo ""
      echo "Add to your PATH (temporary):"
      echo "  export PATH=\"$instdir:\$PATH\""
      echo ""
      if [[ -n "${SHELL:-}" ]]; then
        rcfile=""
        case "${SHELL##*/}" in
          bash) rcfile="$HOME/.bashrc" ;;
          zsh) rcfile="$HOME/.zshrc" ;;
        esac
        if [[ -n "$rcfile" ]]; then
          echo "Persist it by adding to $rcfile:"
          echo "  echo 'export PATH=\"$instdir:\$PATH\"' >> $rcfile"
        fi
      fi
      ;;
  esac

  echo ""
  echo "✅ gam installed at $bindest"
  echo "Run: gam"
}

main "$@"


