---
title: Agents
nav_order: 4
has_children: true
description: Use GAM from Cursor, Claude Code, and other MCP clients
---

# Use GAM from an AI agent

This section is for **Cursor, Claude Code, and other MCP clients**. The agent talks to GAM over stdio (`gam mcp`). You still install the **same CLI**; there is no separate agent package.

Humans who only want the terminal workflow should use [CLI](../cli/) instead.

## What the agent is allowed to do

The agent can list accounts, resolve a repo, attach a project, apply identity (`ensure`), create an account, and (legacy) switch the global host mapping.

The agent **must not**:

- Receive or log a private key or passphrase
- Pass a passphrase as an MCP argument (a native OS dialog opens instead)
- Drive the interactive `gam` menu
- Invent `user.email` — it must use a GAM account

## Two ways to call GAM

| Mode | When |
| --- | --- |
| **MCP** (`gam mcp`) | Preferred in Cursor and Claude Code. Tools are listed in [MCP tools](mcp-tools.md). |
| **CLI `--json`** | Scripts, CI, or an agent without MCP. Same resolution rules. Example: `gam ensure --path /repo --json`. |

Do not ask the model to parse human `gam list` tables if `--json` or MCP is available.

## Setup outline

1. [Install](../install.md) `gam` and confirm `gam --help` includes `mcp`.
2. Point the editor at an **absolute path** to the binary if `gam` is not on the GUI `PATH` (common with Cargo installs).
3. Restart the editor / reload MCP.
4. Follow [Cursor](cursor.md) or [Claude Code](claude-code.md).
5. Before `git fetch` / `push` / `commit`, the agent calls `ensure_identity` with the workspace path.

## Creating accounts from an agent

`add_account` generates an ed25519 key like `gam add --name --email`. By default `use_passphrase` is `true`: a **native OS dialog** asks the human to confirm the passphrase twice. The model only sees the **public** key.

{: .important }
If the dialog is cancelled, the tool returns an error. The agent should tell the human to complete the dialog or rerun `gam add` in a terminal — not to type the passphrase into the chat.
