# Git Account Manager CLI (`gam-cli`)

Manage several Git SSH identities on one machine. Attach a repo to the right `user.name`, `user.email` and SSH key without changing the others.

**Developer docs (Spanish, Diátaxis):** [docs/README.md](docs/README.md)  
**New docs page template:** [docs/TEMPLATE.md](docs/TEMPLATE.md)

## Installation

### macOS & Linux
```bash
curl -fsSL https://raw.githubusercontent.com/lucasgio/gam-master/main/install.sh | bash
```

### Windows (PowerShell)
```powershell
iwr https://raw.githubusercontent.com/lucasgio/gam-master/main/install.ps1 -useb | iex
```

### From source
```bash
git clone https://github.com/lucasgio/gam-master.git
cd gam-master
cargo install --path .
```

## Usage

```bash
gam-cli --help              # examples, commands, --verbose
gam-cli help list           # long help for one command
gam-cli                     # interactive menu
gam-cli add
gam-cli list -v             # aliases, keys, fingerprints
gam-cli status              # active account + this repo
gam-cli attach              # bind current git repo
gam-cli doctor              # diagnose git/ssh/keys
gam-cli switch              # legacy global Host mapping
```

Per-repo identity (`attach`) is the supported flow. `switch` still updates a global `Host <hostname>` block in `~/.ssh/config`.

```bash
git remote set-url origin git@github-work:org/repo.git
```

Existing `~/.ssh/gam_config.json` and keys are kept as-is when you upgrade. See [compatibilidad](docs/explanation/como-funciona.md#compatibilidad-de-config).

## Troubleshooting

If an old alias (`gmc`) shadows the binary:

```bash
type gam-cli
type gmc
```

`gam-cli doctor` reports missing keys, git/ssh in PATH, and the current repo identity.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) and [docs/contributing.md](docs/contributing.md).

```bash
cargo test
cargo run -- --help
```

CI builds on Linux, macOS and Windows. Release tags `vX.Y.Z` publish binaries.
