# Git Account Manager CLI (gam-cli)

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

## Troubleshooting

### "gmc: command not found" or "gmc is an alias for..."
If you use plugins that alias `gmc` (though less common than `gmc`), check with:

```bash
type gmc
```

## Manual Installation (Rust)

### Qué es Git Manager Command
- **Gestiona múltiples identidades SSH de Git** (trabajo, personal, etc.) sin fricción.
- **Aliases por cuenta** (p. ej. `Host github-work`) para evitar conflictos en el mismo host.
- **Configuración local por proyecto** (`gmc attach`) para usar el user.name/email correcto.

### Instalación Manual

1.  **Clonar y compilar**:
    ```bash
    git clone https://github.com/lucasgio/gam-master.git
    cd gam-master
    cargo install --path .
    ```

2.  **Mover el binario (Opcional pero recomendado)**:
    ```bash
    sudo cp ~/.cargo/bin/gmc /usr/local/bin/gmc
    ```

### Uso

#### 1. Agregar una nueva cuenta
```bash
gmc add
```

#### 2. Listar cuentas
```bash
gmc list
```

#### 3. Configurar un repositorio (¡Nuevo!)
```bash
cd /path/to/my-repo
gmc attach
```

#### 4. Borrar todo (Reset)
```bash
gmc reset
```

- Aliases por cuenta: crea `Host <alias>` con `HostName`, `IdentityFile` e `IdentitiesOnly yes`.
  - Usa el alias en tus remotos de Git para separar identidades por host.

```bash
git remote set-url origin git@github-work:org/repo.git
```

- Cambio de cuenta: actualiza un bloque activo `Host <host>` para usar la clave de la cuenta seleccionada.

```bash
gmc switch
```

- Ver configuración: muestra el contenido de `~/.ssh/config` desde el menú.

```bash
gmc
```

(En el menú, elige "📄 View SSH config")

- Limpieza segura: al eliminar una cuenta, quita solo el bloque de esa cuenta en `~/.ssh/config`.

```bash
gmc remove
```

- Validaciones y seguridad: email válido, permisos 600 en clave privada y manejo de overwrite de claves.
- Compatibilidad macOS: añade la clave con `--apple-use-keychain` si aplica.

### Cómo instalarlo
- **Método Recomendado (Binarios):**

  Para instalar la última versión (requiere `curl`):

  ```bash
  ```bash
  # macOS / Linux
  curl -fsSL https://raw.githubusercontent.com/lucasgio/gam-master/main/install.sh | bash
  ```

  Esto descargará el binario adecuado para tu sistema y arquitectura, y lo instalará en `/usr/local/bin` (si tienes permiso sudo) o `$HOME/.local/bin`.

- **Opción B: Compilar desde fuente (Para desarrolladores)**
  Requiere Rust instalado (`rustup.rs`).

  ```bash
  ```bash
  git clone https://github.com/lucasgio/gam-master.git
  cd gam-master
  ./install_from_source.sh
  ```

### Cómo contribuir
1) Haz un fork del repositorio
2) Crea una rama descriptiva: `git checkout -b feat/mi-cambio`
3) Desarrolla y valida localmente:
```bash
cargo build --release
cargo build --release
cargo run --bin gam-cli
```
4) Abre un Pull Request con una descripción clara

Notas:
- El CI ejecuta builds en Linux, macOS y Windows.
- Los binarios publicados en Releases se generan automáticamente al crear un tag `vX.Y.Z`.
****