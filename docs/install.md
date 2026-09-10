---
title: Install
nav_order: 2
description: Install the gam CLI on macOS, Linux, or Windows
---

# Install the CLI
{: .no_toc }

Install **`gam` first**, even if you only plan to use Cursor or Claude Code. The agent server is the same binary (`gam mcp`).

<details open markdown="block">
  <summary>
    Table of contents
  </summary>
  {: .text-delta }
- TOC
{:toc}
</details>

## Requirements

- Git and OpenSSH (`ssh`, `ssh-keygen`, `ssh-add`) on `PATH`
- macOS, Linux, or Windows
- Optional: [Rust](https://rustup.rs/) if you install from source

## Quick install (release binary)

### macOS and Linux

```bash
curl -fsSL https://raw.githubusercontent.com/lucasgio/gam-cli/main/install.sh | bash
```

The script installs `gam` to `/usr/local/bin` and creates a `gam-cli` symlink. You will be asked for `sudo`.

### Windows (PowerShell)

```powershell
iwr https://raw.githubusercontent.com/lucasgio/gam-cli/main/install.ps1 -useb | iex
```

The binary lands in `%USERPROFILE%\.gam\bin`. Add that directory to your user `PATH` if the installer says it is missing.

### Verify

```bash
gam --version
gam --help
```

If `gam --help` does not mention `mcp`, `ensure`, and `doctor`, you are running an old binary. See [Which binary am I running?](#which-binary-am-i-running).

## Install from source

```bash
git clone https://github.com/lucasgio/gam-cli.git
cd gam-cli
cargo install --path . --locked --bin gam
```

Cargo installs to `~/.cargo/bin/gam` (or `%USERPROFILE%\.cargo\bin\gam.exe` on Windows). Put that directory on `PATH`.

To also have the `gam-cli` name:

```bash
ln -sf "$(command -v gam)" "$(dirname "$(command -v gam)")/gam-cli"
```

## Which binary am I running?

On some macOS setups, **zsh aliases `gam` to `git am`**. Cursor and Claude Code do not load your interactive shell aliases, but your terminal might. Check:

```bash
type gam
type gam-cli
command -v gam
gam --version
```

| Result | What to do |
| --- | --- |
| `gam is an alias for git am` | Call the real binary: `/usr/local/bin/gam` or `~/.cargo/bin/gam`. Optionally `unalias gam` in `~/.zshrc` if you do not need `git am` as `gam`. |
| `gam` on PATH but `--help` has no `mcp` | An older install (for example `/usr/local/bin/gam`) is shadowing Cargo. Use the full path, or replace the old file. |
| Cursor cannot start MCP with `"command": "gam"` | GUI apps often lack `~/.cargo/bin`. Use an **absolute path** in MCP config. See [Cursor](agents/cursor.md) and [Claude Code](agents/claude-code.md). |

`gam doctor` reports whether `git` and `ssh` are on `PATH` and whether keys exist.

## After install

**Humans (CLI)** — [First steps](cli/first-steps.md): create an account, attach a repo.

**Agents** — add the MCP server, then [Cursor](agents/cursor.md) or [Claude Code](agents/claude-code.md).

## Uninstall

Remove the binary (`/usr/local/bin/gam`, `/usr/local/bin/gam-cli`, or `~/.cargo/bin/gam`). Config and keys stay in `~/.ssh/` until you delete them yourself (`gam reset --yes` removes GAM-managed accounts and keys).
