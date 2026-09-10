# Tutorial: primeros pasos

<!--
audience: desarrollador que instala GAM por primera vez
type: tutorial
-->

## Qué vas a conseguir

Un binario `gam-cli` en PATH, una cuenta SSH de prueba y la certeza de que `gam-cli list` y `gam-cli doctor` responden.

## Requisitos

- Git y OpenSSH (`ssh`, `ssh-keygen`)
- Rust (`cargo`) si compilas desde fuente; si no, el instalador del README

## Pasos

1. Instala (elige una):

   ```bash
   cargo install --path . --bin gam-cli
   # o el script del README
   ```

2. Comprueba la ayuda:

   ```bash
   gam-cli --help
   gam-cli help list
   ```

   Debes ver ejemplos (`list -v`, `doctor`) y el flag `--verbose`.

3. Crea una cuenta:

   ```bash
   gam-cli add
   ```

   Completa nombre, email, host e identidad Git. Copia la clave pública que imprime el comando.

4. Lista y diagnostica:

   ```bash
   gam-cli list -v
   gam-cli doctor
   ```

## Resultado esperado

- `~/.ssh/gam_config.json` con tu cuenta
- `~/.ssh/id_<nombre>_<host>` y `.pub`
- `gam-cli doctor` sin `key MISSING`

## Si algo falla

| Síntoma | Qué hacer |
|---|---|
| `command not found` | El binario no está en PATH. Revisa `~/.cargo/bin` o `/usr/local/bin`. |
| `ssh-keygen failed` | Permisos en `~/.ssh` o passphrase vacía mal pasada. |
| doctor marca `key MISSING` | La cuenta apunta a un archivo que no existe. `gam-cli list -v` muestra la ruta. |

## Relacionado

- [How-to: varias cuentas](../how-to/usar-varias-cuentas.md)
- [Referencia CLI](../reference/cli.md)
