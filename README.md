# Git Account Manager (`gam`)

Manage several Git SSH identities on one machine. Attach a repo to the right `user.name`, `user.email`, and SSH key, and expose that mapping to AI agents over MCP.

The public command is **`gam`**. Installers also create a `gam-cli` alias.

**Documentation (English):** [https://lucasgio.github.io/gam-cli/](https://lucasgio.github.io/gam-cli/)

| You | Start here |
| --- | --- |
| Terminal user | [Install](https://lucasgio.github.io/gam-cli/install/) → [CLI](https://lucasgio.github.io/gam-cli/cli/) |
| Cursor | [Install](https://lucasgio.github.io/gam-cli/install/) → [Cursor](https://lucasgio.github.io/gam-cli/agents/cursor/) |
| Claude Code | [Install](https://lucasgio.github.io/gam-cli/install/) → [Claude Code](https://lucasgio.github.io/gam-cli/agents/claude-code/) |

## What it does

- Several SSH identities (work, personal, …) with aliases such as `Host github-work`
- **Per-repo attach:** `user.name`, `user.email`, `core.sshCommand`
- **`.gam.json`** in the repo: only `account` and `host_alias` — no emails, keys, or paths
- **`gam ensure`:** what agents and hooks should run. Applies the resolved identity; does not write secrets
- **`gam mcp`:** MCP stdio server in the same binary
- **`gam doctor`:** git / ssh / key diagnostics

`gam switch` still exists (legacy): it changes the global account and the `Host github.com` block in `~/.ssh/config`. For per-project work use `attach` / `ensure`.

## Installation

### macOS and Linux

```bash
curl -fsSL https://raw.githubusercontent.com/lucasgio/gam-cli/main/install.sh | bash
```

### Windows (PowerShell)

```powershell
iwr https://raw.githubusercontent.com/lucasgio/gam-cli/main/install.ps1 -useb | iex
```

### From source

```bash
git clone https://github.com/lucasgio/gam-cli.git
cd gam-cli
cargo install --path . --locked --bin gam
```

On some Macs, zsh aliases `gam` to `git am`. Use `type gam` and the full path (`~/.cargo/bin/gam` or `/usr/local/bin/gam`) if that happens. Details: [install docs](https://lucasgio.github.io/gam-cli/install/).

## CLI usage

```bash
gam --help              # examples, commands, --verbose, --json
gam help list           # long help for one command
gam                     # interactive menu
gam add
gam list -v             # aliases, keys, fingerprints
gam status              # active account + this repo
gam attach --account work
gam ensure
gam doctor              # diagnose git/ssh/keys
gam switch --account work    # legacy global Host mapping
gam mcp                 # MCP server for agents
```

## Agent usage

Configure MCP with an **absolute** path to `gam` (Cursor’s GUI PATH often lacks Cargo). See [Cursor](https://lucasgio.github.io/gam-cli/agents/cursor/) and [Claude Code](https://lucasgio.github.io/gam-cli/agents/claude-code/).

```json
{
  "mcpServers": {
    "gam": {
      "command": "/full/path/to/gam",
      "args": ["mcp"]
    }
  }
}
```

Before `git fetch` / `push` / `commit`, the agent should call `ensure_identity` (or `gam ensure --json`) with the workspace path. Passphrases use a native OS dialog — never an MCP argument.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) and [docs contributing](https://lucasgio.github.io/gam-cli/contributing/).

```bash
cargo test --locked
cargo run -- --help
```

CI builds on Linux, macOS, and Windows. Release tags `vX.Y.Z` publish binaries.
