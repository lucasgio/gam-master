---
title: Cursor
parent: Agents
nav_order: 1
description: Connect GAM as an MCP server in Cursor
---

# Use GAM in Cursor
{: .no_toc }

<!--
audience: developer using Cursor
type: how-to
-->

Expose GAM accounts and `ensure_identity` to Cursor Agent via MCP stdio.

<details open markdown="block">
  <summary>
    Table of contents
  </summary>
  {: .text-delta }
- TOC
{:toc}
</details>

## When to use this

You use Cursor and want the agent to pick the right Git identity (or create an account) without pasting keys into the chat. For Claude Code, see [Claude Code](claude-code.md).

## Requirements

- [Installed](../install.md) `gam` whose `--help` lists `mcp`
- Cursor with MCP enabled
- Absolute path to the binary if Cursor’s environment has no Cargo/`/usr/local/bin` on `PATH`

Find the binary:

```bash
command -v gam || true
ls ~/.cargo/bin/gam /usr/local/bin/gam 2>/dev/null
```

On macOS, if `type gam` says `alias for git am`, use `~/.cargo/bin/gam` or `/usr/local/bin/gam` in the JSON below — never the alias name alone if it is not a real executable.

## Steps

### 1. Add the MCP server

**Project** (this repo only): `.cursor/mcp.json`

**User** (all projects): `~/.cursor/mcp.json`

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

Example on a Mac with a Cargo install:

```json
{
  "mcpServers": {
    "gam": {
      "command": "/Users/you/.cargo/bin/gam",
      "args": ["mcp"]
    }
  }
}
```

{: .warning }
Do not commit a machine-specific absolute path unless the whole team shares that path. Prefer user-level `~/.cursor/mcp.json`, or document the path in your internal wiki.

### 2. Reload MCP

Reload MCP servers in Cursor (Command Palette → MCP / restart). Open a **new** agent chat so tool lists refresh.

### 3. Smoke-test from the agent

Ask the agent to call `list_accounts`. You should see account ids, not private keys.

Then, in a bound repo, ask it to `resolve_project` with the workspace path, then `ensure_identity` before any `git push`.

### 4. Optional: create an account from the agent

Ask Cursor to run `add_account` with `name` and `email`. Approve the OS passphrase dialog on the machine. Add the returned **public** key to GitHub yourself or ask the agent only to show it.

## Expected result

Cursor Settings → MCP shows server `gam` as connected. Tools include `list_accounts`, `resolve_project`, `list_projects`, `ensure_identity`, `attach_project`, `add_account`, `switch_account`.

## If something fails

| Symptom | Likely cause | What to run |
| --- | --- | --- |
| Server spawn error / command not found | Relative `gam` not on Cursor PATH | Absolute `command` path |
| Tools missing `add_account` | Old binary or stale chat | `gam --help`, new chat after reload |
| Passphrase dialog does not appear | Agent passed `use_passphrase: false`, or no GUI | Set `use_passphrase: true`; on Linux install zenity/kdialog |
| Agent uses the wrong Git user | Never called `ensure_identity` | Call it with the workspace `path` |

## Related

- [MCP tools](mcp-tools.md)
- [Claude Code](claude-code.md)
- [CLI `--json` fallback](../cli/commands.md)
