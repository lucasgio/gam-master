---
title: Claude Code
parent: Agents
nav_order: 2
description: Connect GAM as an MCP server in Claude Code
---

# Use GAM in Claude Code
{: .no_toc }

<!--
audience: developer using Claude Code
type: how-to
-->

Expose GAM to Claude Code over MCP stdio so the agent can resolve and apply Git identities.

<details open markdown="block">
  <summary>
    Table of contents
  </summary>
  {: .text-delta }
- TOC
{:toc}
</details>

## When to use this

You use Claude Code (`claude`) in a terminal or IDE and want the same GAM mapping as the CLI. For Cursor, see [Cursor](cursor.md).

## Requirements

- [Installed](../install.md) `gam` with an `mcp` subcommand
- Claude Code CLI
- Absolute path to `gam` recommended (GUI/IDE launches often have a short `PATH`)

```bash
command -v gam
gam --help | head
```

If `gam` is a zsh alias for `git am`, pass the real executable path to Claude Code.

## Steps

### Option A — `claude mcp add`

User scope (all projects):

```bash
claude mcp add --transport stdio gam -- /full/path/to/gam mcp
```

Project scope (this repo):

```bash
claude mcp add --transport stdio --scope project gam -- /full/path/to/gam mcp
```

Replace `/full/path/to/gam` with e.g. `/usr/local/bin/gam` or `$HOME/.cargo/bin/gam`.

### Option B — `.mcp.json` in the project

```json
{
  "mcpServers": {
    "gam": {
      "type": "stdio",
      "command": "/full/path/to/gam",
      "args": ["mcp"]
    }
  }
}
```

User-level config also lives in Claude Code’s MCP settings (`claude mcp list` to inspect).

{: .note }
Commit `.mcp.json` only if the `command` path works for everyone (for example `/usr/local/bin/gam`). Otherwise add the server with user scope.

### Reload and verify

```bash
claude mcp list
```

Start a new Claude Code session. Ask it to use `list_accounts`. Before git network operations, it should call `ensure_identity` with the project path.

## Expected result

`claude mcp list` shows `gam`. Available tools match [MCP tools](mcp-tools.md). Creating an account with `add_account` opens a native passphrase dialog on your desktop.

## If something fails

| Symptom | Likely cause | What to run |
| --- | --- | --- |
| Server not listed | Wrong scope or JSON | `claude mcp list`, check `.mcp.json` |
| `ENOENT` / spawn failed | Alias or missing PATH | Absolute `command` |
| Headless Linux, no passphrase UI | No `DISPLAY` / zenity | Run `gam add --name … --passphrase-prompt` in a graphical session, or `use_passphrase: false` only if you accept an unprotected key |
| Agent skips ensure | Prompt did not require it | Remind: call `ensure_identity` before `git push` |

## Related

- [MCP tools](mcp-tools.md)
- [Cursor](cursor.md)
- [Install](../install.md)
