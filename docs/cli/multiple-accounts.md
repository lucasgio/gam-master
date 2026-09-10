---
title: Multiple accounts
parent: CLI
nav_order: 2
description: Keep work and personal Git identities on one machine
---

# Use several Git accounts
{: .no_toc }

<!--
audience: person who already ran gam add once
type: how-to
-->

Keep a work identity and a personal identity on the same laptop without sharing SSH keys.

<details open markdown="block">
  <summary>
    Table of contents
  </summary>
  {: .text-delta }
- TOC
{:toc}
</details>

## When to use this

You already have one GAM account and need another host user (for example `github.com` work vs personal). If you only need to bind a clone, see [attach a repository](attach-repo.md).

## Requirements

- `gam list` already shows at least one account
- Permission to add a second SSH key on the Git host(s)

## Steps

1. Add the second account:

   ```bash
   gam add --name personal --email you@personal.tld --passphrase-prompt
   ```

   Or run `gam add` and follow the prompts.

2. Add the new **public** key to the *other* GitHub/GitLab user.

3. Attach each clone to the matching account:

   ```bash
   cd ~/src/work-app
   gam attach --account work

   cd ~/src/side-project
   gam attach --account personal
   ```

4. Set remotes to the SSH aliases (not a shared `git@github.com`):

   ```bash
   git remote set-url origin git@github-work:company/work-app.git
   git remote set-url origin git@github-personal:you/side-project.git
   ```

5. Check:

   ```bash
   gam list -v
   gam status
   ```

## Why aliases beat a global switch

`gam switch --account work` rewrites a **global** `Host github.com` block. The next `switch` overwrites it. Two clones of `github.com` then fight over one default key.

Per-repo `attach` plus `Host github-work` / `Host github-personal` keeps keys separate. `IdentitiesOnly yes` stops ssh from offering the other keys.

## Expected result

Each repo has its own `.gam.json` and local `user.email`. `ssh -T git@github-work` and `ssh -T git@github-personal` authenticate as different users.

## If something fails

| Symptom | Likely cause | What to run |
| --- | --- | --- |
| Both remotes still use `github.com` | Origin URL not updated | `git remote -v` then `set-url` with the alias |
| ssh offers the wrong key | Missing `IdentitiesOnly` or using `switch` only | `gam list -v`, inspect `~/.ssh/config` |
| `overwrite` needed | Re-running `add` with the same name | `gam add --name work --email … --overwrite` only if you intend to replace the key |

## Related

- [First steps](first-steps.md)
- [How GAM works](../explanation.md)
