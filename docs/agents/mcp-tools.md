---
title: MCP tools
parent: Agents
nav_order: 3
description: MCP tool names, arguments, and side effects
---

# MCP tool reference
{: .no_toc }

<!--
audience: agent author or developer wiring MCP
type: reference
-->

Transport: stdio. Start: `gam mcp`. Private keys and passphrases are never returned.

<details open markdown="block">
  <summary>
    Table of contents
  </summary>
  {: .text-delta }
- TOC
{:toc}
</details>

## Tools

| Tool | Side effects | Arguments |
| --- | --- | --- |
| `list_accounts` | None | — |
| `resolve_project` | None (read-only) | `path?` — git work tree; default process cwd |
| `list_projects` | None | — |
| `ensure_identity` | Writes local git config; updates path registry. Does **not** write `.gam.json` | `path?`, `account?` |
| `attach_project` | Writes `.gam.json`, local git config, registry | `account` (required), `path?` |
| `add_account` | Creates key files, `gam_config.json`, optional `~/.ssh/config`, ssh-agent. Returns **public** key only | See below |
| `switch_account` | Legacy global `Host` remap | `account` (required) |

Call **`ensure_identity`** with the workspace path before `git fetch`, `git push`, or `git commit`.

Resolution order when `account` is omitted: `.gam.json` → local registry → `origin` URL → `user.email` → `no_mapping` error with the account list.

## `add_account` arguments

| Field | Required | Default |
| --- | --- | --- |
| `name` | Yes | — |
| `email` | Yes | — |
| `host` | No | `github.com` |
| `description` | No | — |
| `git_user_name` | No | `name` |
| `git_user_email` | No | `email` |
| `overwrite` | No | `false` |
| `update_ssh_config` | No | `true` |
| `use_passphrase` | No | **`true`** |

There is **no** passphrase field. When `use_passphrase` is true, GAM opens a native OS dialog and confirms twice. Set `use_passphrase` to `false` only to generate an unprotected key.

## JSON errors

Failed tools return JSON with `ok: false` and a code such as `no_mapping`. The CLI `--json` flag uses the same shapes.

## Related

- [Cursor](cursor.md)
- [Claude Code](claude-code.md)
- [CLI commands](../cli/commands.md)
