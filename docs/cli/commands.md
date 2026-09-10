---
title: Commands
parent: CLI
nav_order: 4
description: CLI command and flag reference
---

# CLI command reference
{: .no_toc }

<!--
audience: human or contributor looking up flags
type: reference
-->

Binary: **`gam`** (installers also add `gam-cli`). No subcommand opens the interactive menu.

<details open markdown="block">
  <summary>
    Table of contents
  </summary>
  {: .text-delta }
- TOC
{:toc}
</details>

## Global flags

| Flag | Effect |
| --- | --- |
| `-v`, `--verbose` | Extra detail on `list`, `status`, `doctor` (fingerprints, `Host` blocks) |
| `--json` | JSON on stdout; non-interactive |
| `-h`, `--help` | Short help; `gam help <command>` is long help with examples |
| `--version` | Crate version |

## Commands

| Command | What it does | Notes |
| --- | --- | --- |
| `add` | Create account + ed25519 key + SSH `Host` alias | Interactive, or `--name` + `--email`. `--passphrase-prompt` opens a native OS dialog. `--json` prints a public-key payload, never the private key. `--overwrite` replaces an existing account. `--host` default `github.com`. `--update-ssh-config` default true. |
| `list` | List accounts | `-v`: fingerprint, `.pub` path, presence in `~/.ssh/config`. `--json` omits private key paths. |
| `status` | Global active account + this repo | `--json` |
| `attach` | Local identity + write `.gam.json` | `--account`, `--path` |
| `ensure` | Apply resolved identity; do **not** write `.gam.json` | Preferred for agents and hooks. `--account`, `--path` |
| `resolve` | Read-only: which account this repo would use | `--path`, `--json` |
| `projects` | Local path / origin / account registry | `--json` |
| `switch` | Legacy global active account + `Host <hostname>` remap | Prefer `attach` / `ensure` |
| `doctor` | Diagnose git/ssh/keys/repo identity | `-v` full card. `--json`. Exit `1` if issues (except “no accounts yet”). |
| `mcp` | MCP stdio server | Same binary; see [Agents](../agents/) |
| `remove` | Delete one account, key, and alias block | `--account`, `--yes` when non-interactive |
| `reset` | Delete **all** GAM-managed accounts and keys | `--yes` |

### `gam add` flags

| Flag | Required | Default |
| --- | --- | --- |
| `--name` | Yes unless interactive | — |
| `--email` | Yes unless interactive | — |
| `--host` | No | `github.com` |
| `--description` | No | — |
| `--git-user-name` | No | account name |
| `--git-user-email` | No | `--email` |
| `--overwrite` | No | false |
| `--update-ssh-config` | No | true |
| `--passphrase-prompt` | No | false on CLI (interactive `add` still asks). MCP `add_account` defaults to **true**. |

Passphrase dialogs: macOS `osascript`, Windows PowerShell `Get-Credential`, Linux `zenity` → `kdialog` → `yad` → `SSH_ASKPASS`. Linux needs `DISPLAY` or `WAYLAND_DISPLAY`. The secret is never written to JSON or MCP logs.

## SSH alias rule

`Host` = `{first-label-of-host}-{account-name}` with spaces in the name replaced by `-`. Example: host `github.com` + account `work` → `github-work`.

## Embedded help

`about` / `after_help` live in `src/main.rs`. Console helpers live in `src/ui.rs`. If you change a command, update this page in the same PR. The test `root_help_includes_examples_and_doctor` fails if `--help` drops examples or `doctor`.

## Related

- [Files and exit codes](../reference/)
- [MCP tools](../agents/mcp-tools.md)
- [Documentation template](../contributing/template.md)
