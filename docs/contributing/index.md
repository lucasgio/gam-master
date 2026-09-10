---
title: Contributing
nav_order: 7
has_children: true
description: Build, test, and document GAM
---

# Contributing

## Code

```bash
cargo build --locked
cargo test --locked
cargo run -- --help
cargo run -- list -v
cargo run -- doctor
```

Unit tests should not require a real `~/.ssh` tree.

CI builds on Linux, macOS, and Windows. Tags `vX.Y.Z` publish release binaries.

## Documentation

Docs are English, built with [Just the Docs](https://just-the-docs.github.io/just-the-docs/), and follow [Diátaxis](https://diataxis.fr/).

1. Choose tutorial, how-to, reference, or explanation — see the [template](template.md).
2. Put the page in `cli/` or `agents/` (or `explanation.md` / `reference.md`).
3. Fill YAML front matter (`title`, `parent`, `nav_order`).
4. If you changed flags, tools, or files, update [CLI commands](../cli/commands.md) and/or [MCP tools](../agents/mcp-tools.md) in the **same PR**.
5. The test `root_help_includes_examples_and_doctor` fails if you drop examples or `doctor` from `--help`.

Preview locally:

```bash
cd docs
bundle install
bundle exec jekyll serve --baseurl /gam-cli
```

Open `http://127.0.0.1:4000/gam-cli/`.

## CLI style

- Command messages are English.
- Documentation is English.
- Empty states should say **what to do next**.
- Extra noise (fingerprints, SSH blocks) stays behind `-v`, except `status` / `doctor`.

## Security

Never document or commit private keys, passphrases, or a real `gam_config.json`. MCP and `--json` must not echo secrets.

## Pull requests

Descriptive branch, green `cargo test`, and a description that links the docs page if behavior changed. Default branch is `main`.
