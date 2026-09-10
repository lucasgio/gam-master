---
title: Reference
nav_order: 6
has_children: false
description: Files, resolution order, and exit codes
---

# Files, resolution, and exit codes
{: .no_toc }

<!--
audience: developer or contributor
type: reference
-->

<details open markdown="block">
  <summary>
    Table of contents
  </summary>
  {: .text-delta }
- TOC
{:toc}
</details>

## Files

| Path | Contents |
| --- | --- |
| `~/.ssh/gam_config.json` | `accounts`, `current_account`, `projects` |
| `~/.ssh/ssh_manager_config.json` | Legacy config; copied to `gam_config.json` if the new file is absent |
| `~/.ssh/id_<account>_<host>` | Private key |
| `~/.ssh/id_<account>_<host>.pub` | Public key |
| `~/.ssh/config` | Per-account `Host` aliases and optional `# gam ACTIVE START` block |
| `<repo>/.gam.json` | `account` + `host_alias` only (no secrets) |
| `<repo>/.git/config` | Local identity after `attach` / `ensure` |

Account fields: `name`, `email`, `key_file`, `host`, `description`, `git_user_name`, `git_user_email` (the last two default empty when deserializing).

`.gam.json` example:

```json
{
  "account": "work",
  "host_alias": "github-work"
}
```

## Resolution order

Used by `ensure`, `resolve`, and MCP `ensure_identity` / `resolve_project` when `account` is omitted:

1. Explicit `--account` / tool `account`
2. `.gam.json` in the repo root
3. Local path registry (`projects` in `gam_config.json`)
4. `origin` URL (SSH alias or host), even if the local path is new
5. Local `user.email` matching an account
6. `no_mapping` plus the account list

## Exit codes

| Code | Meaning |
| --- | --- |
| `0` | Operation succeeded (`ok: true` in JSON) |
| `1` | `no_mapping`, missing account, doctor issues (except when there are no accounts yet), or other errors |

`--json` prints the error object on stdout.

## Related

- [CLI commands](cli/commands.md)
- [MCP tools](agents/mcp-tools.md)
