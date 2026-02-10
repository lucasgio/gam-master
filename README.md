# Git Account Manager (gam)

A simple CLI tool to manage multiple Git SSH accounts, now with per-project git configuration.

## Installation

### macOS & Linux
```bash
curl -fsSL https://raw.githubusercontent.com/lucasgio/gam-master/main/install.sh | bash
```

### Windows (PowerShell)
```powershell
iwr https://raw.githubusercontent.com/lucasgio/gam-master/main/install.ps1 -useb | iex
```

## Manual Installation (Rust)

### Qué es Git Account Manager
- **Gestiona múltiples identidades SSH de Git** (trabajo, personal, etc.) sin fricción.
- **Aliases por cuenta** (p. ej. `Host github-work`) para evitar conflictos en el mismo host.
- **Cambio rápido de identidad activa** por host (actualiza `~/.ssh/config` de forma segura).
- **Generación de claves ED25519** con passphrase opcional e integración con macOS Keychain.


### Funcionalidades
- Generar claves SSH (ED25519) con passphrase opcional e instalación en ssh-agent/Keychain.

```bash
gam add
```

- Gestionar cuentas: agregar, listar, cambiar activa, eliminar.

```bash
gam add
gam list
gam switch
gam remove
```

- Aliases por cuenta: crea `Host <alias>` con `HostName`, `IdentityFile` e `IdentitiesOnly yes`.
  - Usa el alias en tus remotos de Git para separar identidades por host.

```bash
git remote set-url origin git@github-work:org/repo.git
```

- Cambio de cuenta: actualiza un bloque activo `Host <host>` para usar la clave de la cuenta seleccionada.

```bash
gam switch
```

- Ver configuración: muestra el contenido de `~/.ssh/config` desde el menú.

```bash
gam
```

(En el menú, elige "📄 View SSH config")

- Limpieza segura: al eliminar una cuenta, quita solo el bloque de esa cuenta en `~/.ssh/config`.

```bash
gam remove
```

- Validaciones y seguridad: email válido, permisos 600 en clave privada y manejo de overwrite de claves.
- Compatibilidad macOS: añade la clave con `--apple-use-keychain` si aplica.

### Cómo instalarlo
- **Método Recomendado (Binarios):**

  Para instalar la última versión (requiere `curl`):

  ```bash
  # macOS / Linux
  curl -fsSL https://raw.githubusercontent.com/giolabs/gam/main/install.sh | bash
  ```

  Esto descargará el binario adecuado para tu sistema y arquitectura, y lo instalará en `/usr/local/bin` (si tienes permiso sudo) o `$HOME/.local/bin`.

- **Opción B: Compilar desde fuente (Para desarrolladores)**
  Requiere Rust instalado (`rustup.rs`).

  ```bash
  git clone https://github.com/giolabs/gam.git
  cd gam
  ./install_from_source.sh
  ```

### Cómo contribuir
1) Haz un fork del repositorio
2) Crea una rama descriptiva: `git checkout -b feat/mi-cambio`
3) Desarrolla y valida localmente:
```bash
cargo build --release
cargo run --bin gam
```
4) Abre un Pull Request con una descripción clara

Notas:
- El CI ejecuta builds en Linux, macOS y Windows.
- Los binarios publicados en Releases se generan automáticamente al crear un tag `vX.Y.Z`.
****