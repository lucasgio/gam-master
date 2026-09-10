# Contribuir

<!--
audience: desarrollador que abre un PR
type: how-to
-->

## Código

```bash
cargo build
cargo test
cargo run -- --help
cargo run -- list -v
cargo run -- doctor
```

No hace falta tocar `~/.ssh` real para los tests unitarios de help y alias.

## Documentación

1. ¿Es tutorial, how-to, referencia o explicación? Ver [TEMPLATE.md](TEMPLATE.md).
2. Copia la plantilla a la carpeta correcta.
3. Enlázala desde [README.md](README.md).
4. Si cambiaste flags o comandos, actualiza [reference/cli.md](reference/cli.md) **en el mismo PR**.
5. El test `root_help_includes_examples_and_doctor` falla si quitas ejemplos o `doctor` del `--help`.

## Estilo del CLI

- Mensajes de comando en inglés (como el resto del binario).
- Docs en español.
- Estados vacíos deben decir **qué hacer después** (`ui::empty_state` / `ui::next_steps`).
- Lo extra (fingerprints, bloques SSH) va detrás de `-v`, no en el default ruidoso — excepto `status`/`doctor`, que existen para diagnosticar.

## PR

Rama descriptiva, `cargo test` verde, descripción que enlace la página de docs si cambia el comportamiento.
