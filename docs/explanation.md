---
title: How GAM works
nav_order: 5
description: Why per-repo attach and SSH host aliases exist
---

# How GAM works
{: .no_toc }

<!--
audience: contributor or advanced user
type: explanation
-->

<details open markdown="block">
  <summary>
    Table of contents
  </summary>
  {: .text-delta }
- TOC
{:toc}
</details>

## The problem

Git has an identity (`user.name` / `user.email`). OpenSSH has keys. One laptop often has a work user and a personal user against the same `github.com`. A single `Host github.com` entry in `~/.ssh/config` is not enough: the last key wins, and every clone inherits it.

## Two layers (plus a legacy third)

1. **GAM account** — metadata and key path in `~/.ssh/gam_config.json`.
2. **Repository** — `attach` writes **local** git config and a portable `.gam.json`. Other clones do not change.
3. **`switch` (legacy)** — rewrites a marked `Host github.com` (or other hostname) block between `# gam ACTIVE START` comments. Global. Unsafe when two clones share that host.

Agents should use layer 2 (`ensure_identity` / `attach_project`), not `switch_account`, unless they are reproducing the old global workflow on purpose.

## Alias vs active mapping

Creating an account can add:

```
Host github-work
    HostName github.com
    IdentityFile ~/.ssh/id_work_github_com
    IdentitiesOnly yes
```

Remote `git@github-work:org/repo.git` selects that key. `IdentitiesOnly` stops ssh from trying the others.

`attach` / `ensure` also set `core.sshCommand` locally so even an origin that still says `github.com` can use the right `-i` key.

## Config compatibility

On startup GAM reads, in order:

1. `~/.ssh/gam_config.json`
2. If missing, `~/.ssh/ssh_manager_config.json` (copied to the new path)

New `SshAccount` fields use `#[serde(default)]`. An old JSON without `git_user_name` still loads. There is no destructive migration: existing keys and already-attached repos are not rewritten.

A previous `attach` remains in `.git/config` after you upgrade the binary.

## What lives in the code

| Area | Files |
| --- | --- |
| CLI parsing | `src/main.rs` |
| Account/repo operations | `src/manager.rs`, `src/config.rs`, `src/git.rs`, `src/ssh.rs` |
| MCP | `src/mcp.rs` |
| Passphrase dialogs | `src/passphrase.rs` |
| Console copy | `src/ui.rs` |

Clap `long_about` / `after_help` is in-terminal help. This `docs/` site is for humans (and agents) reading the web.

## Related

- [CLI commands](cli/commands.md)
- [MCP tools](agents/mcp-tools.md)
- [Files and exit codes](reference/)
