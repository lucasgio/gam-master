# Changelog

All notable changes to this project will be documented in this file.

## [v0.4.0] – 2026-09-10

### Added

- MCP stdio server in the same `gam` binary (`gam mcp`) with tools for list/resolve/ensure/attach/add/switch
- Agent-oriented CLI: `--json`, `--account`, `--path`, `--yes`, plus `ensure`, `resolve`, and `projects`
- `add_account` / `gam add --name --email` with a native OS passphrase dialog (secret never appears in JSON or MCP)
- Portable `.gam.json` (`account` + `host_alias` only) written by `attach`
- English GitHub Pages documentation (Just the Docs), with CLI and agent (Cursor / Claude Code) sections split

### Changed

- Canonical binary name is `gam` (`gam-cli` remains an installer alias)
- Installers download from `lucasgio/gam-cli`
- `--help` points at https://lucasgio.github.io/gam-cli/

### Fixed

- `scripts/install.sh` asset names now match the GitHub Release workflow
