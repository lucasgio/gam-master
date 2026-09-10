# Documentación de GAM (`gam-cli`)

Guía para **desarrolladores que usan o contribuyen** a Git Account Manager.

La documentación sigue [Diátaxis](https://diataxis.fr/): tutoriales, how-to, referencia y explicación. Para añadir una página nueva copia [`TEMPLATE.md`](TEMPLATE.md).

## Empieza aquí

| Si quieres… | Lee |
|---|---|
| Instalar y crear la primera cuenta | [Tutorial: primeros pasos](tutorials/01-primeros-pasos.md) |
| Trabajar con trabajo + personal en el mismo laptop | [How-to: varias cuentas](how-to/usar-varias-cuentas.md) |
| Atar un repo a una identidad | [How-to: attach](how-to/adjuntar-un-repo.md) |
| Ver todos los comandos y flags | [Referencia CLI](reference/cli.md) |
| Entender el modelo (config, SSH, attach vs switch) | [Cómo funciona](explanation/como-funciona.md) |
| Contribuir código o docs | [Contribuir](contributing.md) |

## Mapa Diátaxis

```
tutorials/     aprender haciendo (primera vez)
how-to/        resolver una tarea concreta
reference/     qué existe (comandos, archivos, flags)
explanation/   por qué está diseñado así
```

## Código y config en runtime

- Binario: `gam-cli` (menú interactivo si no pasas subcomando)
- Config de cuentas: `~/.ssh/gam_config.json`
- Claves: `~/.ssh/id_<cuenta>_<host>`
- Identidad por repo: `git config --local` (`user.name`, `user.email`, `core.sshCommand`)

Las cuentas que ya tienes **no se migran ni se borran** al actualizar el CLI. Ver [cómo funciona](explanation/como-funciona.md#compatibilidad-de-config).
