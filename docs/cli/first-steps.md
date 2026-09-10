---
title: First steps
parent: CLI
nav_order: 1
description: Create an account, publish the public key, and attach a repository
---

# First steps with the CLI
{: .no_toc }

<!--
audience: person who just installed gam
type: tutorial
-->

You will create one SSH account, add its public key to your Git host, and bind a local clone so commits use the right name, email, and key.

<details open markdown="block">
  <summary>
    Table of contents
  </summary>
  {: .text-delta }
- TOC
{:toc}
</details>

## When to use this

You have [installed](../install.md) `gam` and want a working identity on one repository. For a second job/personal account, continue with [multiple accounts](multiple-accounts.md).

## Requirements

- `gam` on your `PATH` (or the absolute path from `type gam`)
- A Git host account (GitHub, GitLab, …) where you can add an SSH key
- A local git clone you can configure

## Steps

### 1. Create an account

Interactive (TTY):

```bash
gam add
```

Answer name, email, host (default `github.com`), and passphrase prompts.

Non-interactive, with a native OS passphrase dialog (the secret is not printed):

```bash
gam add --name work --email you@company.com --passphrase-prompt
```

GAM generates an ed25519 key, writes `~/.ssh/gam_config.json`, adds a `Host` alias such as `github-work`, and prints the **public** key.

{: .warning }
Never paste a private key into chat, tickets, or git. Only the `.pub` line goes to the Git host.

### 2. Register the public key

Copy the printed `ssh-ed25519 …` line (or `cat ~/.ssh/id_work_github_com.pub`) into GitHub → Settings → SSH keys, or the equivalent on your host.

### 3. Attach a repository

```bash
cd /path/to/your/repo
gam attach --account work
```

This writes:

- `.gam.json` — `account` and `host_alias` only (safe to commit)
- local `.git/config` — `user.name`, `user.email`, `core.sshCommand`

### 4. Point origin at the SSH alias (recommended)

```bash
git remote set-url origin git@github-work:org/repo.git
gam status
```

Replace `github-work` with the alias `gam attach` printed (also in `.gam.json` as `host_alias`).

### 5. Confirm Git can authenticate

```bash
gam doctor
ssh -T git@github-work
```

`doctor` should not report missing keys. The SSH test should authenticate as the user that owns the key.

## Expected result

- `gam list` shows the new account
- `gam status` in the repo shows that account and matching `user.email`
- `.gam.json` looks like:

```json
{
  "account": "work",
  "host_alias": "github-work"
}
```

## If something fails

| Symptom | Likely cause | What to run |
| --- | --- | --- |
| `gam` runs `git am` | Shell alias | [Install: which binary](../install.md#which-binary-am-i-running) |
| Permission denied (publickey) | Key not on the host, or wrong `Host` | `gam list -v`, `ssh -T git@<alias>` |
| Commits have the wrong email | Repo not attached | `gam attach --account work` then `git config --local --get user.email` |
| Passphrase dialog never appears | Linux without GUI helper | Install `zenity`, `kdialog`, or `yad`, or use interactive `gam add` |

## Related

- [Attach a repository](attach-repo.md)
- [CLI commands](commands.md)
- [How GAM works](../explanation.md)
