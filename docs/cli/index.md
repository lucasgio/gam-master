---
title: CLI
nav_order: 3
has_children: true
description: Use GAM from a terminal — interactive menu and commands
---

# Use GAM from the CLI

This section is for **you at a keyboard**. You run `gam` in a terminal, answer prompts, and inspect status with `list` / `status` / `doctor`.

If you want an AI assistant to apply identities, go to [Agents](../agents/) instead. Agents should not be taught to drive the interactive menu.

## Typical loop

1. [Install](../install.md) `gam`.
2. Create an account: `gam add` (or follow [first steps](first-steps.md)).
3. Add the printed **public** key to GitHub, GitLab, or your Git host.
4. In a clone: `gam attach --account <id>` — writes `.gam.json` and local git config.
5. Optionally point `origin` at the SSH alias: `git@github-work:org/repo.git`.
6. Day to day: `gam status`, `gam doctor` if SSH or identity looks wrong.

`gam switch` still exists. It remaps a **global** `Host github.com` (or other host) block. Prefer `attach` so each repo keeps its own identity.

## Global flags

| Flag | Meaning |
| --- | --- |
| `-v`, `--verbose` | Extra diagnostics on `list`, `status`, `doctor` |
| `--json` | Machine-readable stdout; implies non-interactive |
| `-h`, `--help` | Short help; `gam help <command>` is the long form with examples |
| `--version` | Crate version |

`--json` is the scripting interface. MCP is the agent interface. You can use `--json` from agents if MCP is not configured; prefer MCP when it is.

## Next

- Tutorial: [First steps](first-steps.md)
- How-to: [Multiple accounts](multiple-accounts.md), [Attach a repository](attach-repo.md)
- Reference: [Commands](commands.md)
