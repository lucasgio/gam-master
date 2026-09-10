---
title: Attach a repository
parent: CLI
nav_order: 3
description: Bind a git clone to a GAM account and write .gam.json
---

# Attach a repository
{: .no_toc }

<!--
audience: person using GAM in a clone
type: how-to
-->

Bind the current git work tree to an account so local commits and SSH use that identity.

<details open markdown="block">
  <summary>
    Table of contents
  </summary>
  {: .text-delta }
- TOC
{:toc}
</details>

## When to use this

You already have the account in `gam list` and you are inside (or can pass `--path` to) a git clone. Agents should call the MCP tool `attach_project` instead of this page’s interactive flow — see [MCP tools](../agents/mcp-tools.md).

## `attach` vs `ensure`

| Command | Writes `.gam.json` | Applies local git identity | Updates path registry |
| --- | --- | --- | --- |
| `gam attach --account <id>` | Yes | Yes | Yes |
| `gam ensure` | No | Yes | Yes |

Use **attach** once per clone (and commit `.gam.json` if the team should share the mapping). Use **ensure** in hooks or before git network commands when `.gam.json` or the registry already exists.

## Steps

1. Confirm the account id:

   ```bash
   gam list
   ```

2. Attach:

   ```bash
   cd /path/to/repo
   gam attach --account work
   ```

   From another directory:

   ```bash
   gam attach --account work --path /path/to/repo
   ```

3. Optionally rewrite `origin`:

   ```bash
   git remote set-url origin git@<host_alias>:org/repo.git
   ```

4. Verify:

   ```bash
   cat .gam.json
   git config --local --get user.email
   gam status
   ```

## Expected result

`.gam.json`:

```json
{
  "account": "work",
  "host_alias": "github-work"
}
```

Local git config has `user.name`, `user.email`, and `core.sshCommand` pointing at the account key.

## If something fails

| Symptom | Likely cause | What to run |
| --- | --- | --- |
| Missing `--account` and no TTY | Non-interactive without an id | Pass `--account` |
| `no_mapping` on `ensure` | Never attached, no `.gam.json`, origin does not match | `gam attach --account …` |
| Wrong email after attach | Account `git_user_email` vs key comment email | `gam list -v`, re-attach |

## Related

- [CLI commands](commands.md)
- [Files and exit codes](../reference/)
