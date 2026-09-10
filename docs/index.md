---
title: Home
nav_order: 1
description: Git Account Manager — one binary for humans in the terminal and for AI agents over MCP
permalink: /
---

# Git Account Manager
{: .fs-9 }

One binary (`gam`) for Git SSH identities — in the terminal, and inside Cursor or Claude Code.
{: .fs-6 .fw-300 }

[Install the CLI](install.md){: .btn .btn-primary .fs-5 .mb-4 .mb-md-0 .mr-2 }
[Use from an agent](agents/){: .btn .fs-5 .mb-4 .mb-md-0 }

---

GAM keeps several Git identities on one machine. Each account has its own ed25519 key, SSH `Host` alias, and `user.name` / `user.email`. You attach a **repository** to an account. AI agents reuse the same mapping over MCP — they never receive private keys or passphrases.

The public command is **`gam`**. Installers also create a `gam-cli` alias.

## Pick a path

| I work in a terminal | I work with an AI agent |
| --- | --- |
| Interactive `gam` menu, `add`, `attach`, `status`, `doctor` | Same binary: `gam mcp`, plus `gam ensure --json` |
| You type commands and confirm prompts | Cursor or Claude Code call MCP tools |
| Start with [Install](install.md) then [CLI first steps](cli/first-steps.md) | Start with [Install](install.md) then [Agents](agents/) |

Do **not** mix the two mental models. The CLI is for you. The agent surface is MCP tools (or `--json` if you script without MCP). Agents should call `ensure_identity` / `gam ensure` before `git fetch`, `push`, or `commit` — they should not open the interactive menu.

## What GAM stores

| Location | What is there | Safe to commit? |
| --- | --- | --- |
| `~/.ssh/gam_config.json` | Accounts, current account, local project registry | No |
| `~/.ssh/id_<account>_<host>` | Private key | **Never** |
| `~/.ssh/config` | `Host` aliases and a legacy global `Host` block | No |
| `<repo>/.gam.json` | `account` + `host_alias` only | Yes |
| `<repo>/.git/config` | Local `user.name`, `user.email`, `core.sshCommand` | No (git metadata) |

`.gam.json` never contains emails, key paths, or secrets.

## Account resolution

When GAM decides which account a repo uses (CLI `--account` omitted, or MCP `path` only):

1. `--account` / tool `account` argument
2. `.gam.json` in the repo root
3. Local path registry in `gam_config.json`
4. `origin` URL (SSH alias / host)
5. Local `user.email` that matches an account
6. Error `no_mapping` plus the list of accounts

`gam attach` writes `.gam.json` and applies identity. `gam ensure` applies identity and does **not** write `.gam.json`.

## Docs map

This site follows [Diátaxis](https://diataxis.fr/). New pages start from the [documentation template](contributing/template.md).

| Kind | Pages |
| --- | --- |
| Tutorial | [CLI first steps](cli/first-steps.md) |
| How-to | [Install](install.md), [multiple accounts](cli/multiple-accounts.md), [attach a repo](cli/attach-repo.md), [Cursor](agents/cursor.md), [Claude Code](agents/claude-code.md) |
| Reference | [CLI commands](cli/commands.md), [MCP tools](agents/mcp-tools.md), [files and exit codes](reference/) |
| Explanation | [How GAM works](explanation.md) |
