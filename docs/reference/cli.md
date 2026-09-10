# Referencia CLI

<!--
audience: desarrollador o contribuidor
type: reference
-->

Binario: `gam-cli`. Sin subcomando abre el menú interactivo.

## Flags globales

| Flag | Efecto |
|---|---|
| `-v`, `--verbose` | Más detalle en `list`, `status`, `doctor` (fingerprints, bloque `Host`, etc.) |
| `-h`, `--help` | Ayuda corta; `gam-cli help <comando>` muestra la larga con ejemplos |
| `--version` | Versión del crate |

## Comandos

| Comando | Qué hace | Notas |
|---|---|---|
| `add` | Crea cuenta + clave ed25519 + (opcional) bloque SSH | Interactivo |
| `list` | Lista cuentas ordenadas | `-v`: fingerprint, `.pub`, presencia en `~/.ssh/config` |
| `status` | Cuenta global activa, test SSH, identidad del cwd | Siempre muestra el repo actual si es git |
| `attach` | `git config --local` de name, email y `core.sshCommand` | Hay que estar en un repo |
| `switch` | Cuenta global + bloque ACTIVE `Host <hostname>` | Legado; afecta a todos los remotes de ese host |
| `doctor` | Diagnóstico: git/ssh en PATH, claves, repo | `-v` imprime la ficha completa de cada cuenta |
| `remove` | Borra cuenta, clave y bloque alias | Pide confirmación |
| `reset` | Borra **todas** las cuentas gestionadas | Irreversible |

## Archivos

| Path | Contenido |
|---|---|
| `~/.ssh/gam_config.json` | `accounts`, `current_account` |
| `~/.ssh/ssh_manager_config.json` | Config antigua; se copia a `gam_config.json` si la nueva no existe |
| `~/.ssh/id_<cuenta>_<host>` | Clave privada |
| `~/.ssh/config` | `Host <alias>` por cuenta y bloque `# gam ACTIVE START` |
| `<repo>/.git/config` | Identidad local tras `attach` |

Campos de cuenta: `name`, `email`, `key_file`, `host`, `description`, `git_user_name`, `git_user_email` (los dos últimos opcionales; default vacío al deserializar).

## Alias SSH

`Host` = `{primer-label-del-host}-{nombre}` con espacios en nombre sustituidos por `-`. Ejemplo: host `github.com` + cuenta `work` → `github-work`.

## Códigos de salida

Hoy los flujos interactivos suelen devolver `0` incluso en cancelaciones (imprimen el error y siguen). Un fallo de I/O o de `ssh-keygen` sí propaga error (`anyhow`). No dependas de códigos distintos de 0/1 hasta que el CLI tenga modo no interactivo.

## Ayuda embebida

Los textos `about` / `after_help` viven en `src/main.rs` (clap) y los helpers de consola en `src/ui.rs`. Si cambias un comando, actualiza esta página y los ejemplos de `--help` (hay un test que exige que el help mencione `doctor` y `Examples`).
