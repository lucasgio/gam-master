---
title: Documentation template
parent: Contributing
nav_order: 1
description: Copy this structure when adding a GAM docs page
---

# Documentation template

Copy the Markdown below into the correct Diátaxis folder (`cli/` or `agents/`, or a top-level explanation/reference page). Delete headings that do not apply. Do not leave empty sections.

**Pick one type:**

- **Tutorial** — the reader is new; one happy path with a visible result
- **How-to** — the reader already uses GAM and wants to finish a task
- **Reference** — facts: flags, files, exit codes. No narrative
- **Explanation** — context and decisions. Not a recipe

```markdown
---
title: <short title>
parent: CLI          # or Agents, Contributing, …
nav_order: 1
description: <one line for search and SEO>
---

# <one-line title>
{: .no_toc }

<!--
audience: (GAM user / contributor / agent integrator)
type: tutorial | how-to | reference | explanation
-->

<details open markdown="block">
  <summary>
    Table of contents
  </summary>
  {: .text-delta }
- TOC
{:toc}
</details>

## When to use this

Short list of situations. Link away if this is the wrong page.

## Requirements

- `gam` on PATH (or absolute path for MCP)
- …

## Steps

1. …
2. …

Show real commands. Sample output must match the current CLI (`-v`, `doctor`, `mcp`, …).

## Expected result

What should appear on disk (`~/.ssh/gam_config.json`, `.git/config`, `.gam.json`) or in the editor (MCP tools connected).

## If something fails

Symptom → likely cause → command (`gam doctor`, `gam list -v`).

## Related

- [other page](…)
```

## Conventions

- One topic per file. Filename: `kebab-case.md`.
- Commands use `gam …`. The interactive menu is `gam` with no subcommand. Mention `gam-cli` only as an installer alias.
- Never document private keys or paste a real `gam_config.json`.
- Language: **English**. Flag names stay as in the binary (`--verbose`, `--json`).
- Split **CLI** vs **Agents**. Do not teach the interactive menu on agent pages; do not bury MCP setup inside CLI tutorials.
- After changing a command or MCP tool, update [CLI commands](../cli/commands.md) and/or [MCP tools](../agents/mcp-tools.md) in the same PR.

## Front matter (Just the Docs)

| Key | Purpose |
| --- | --- |
| `title` | Nav label; must match `parent` on children exactly |
| `parent` | Parent page `title` |
| `nav_order` | Sibling sort |
| `has_children` | Section landing pages |
| `description` | SEO / search snippet |
