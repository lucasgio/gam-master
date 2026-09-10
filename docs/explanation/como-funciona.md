# Cómo funciona GAM

<!--
audience: contribuidor o usuario avanzado
type: explanation
-->

## El problema

Git usa una identidad (`user.name` / `user.email`) y OpenSSH una clave. En un mismo laptop es habitual tener cuenta de trabajo y personal contra `github.com`. Un solo `Host github.com` en `~/.ssh/config` no basta: la última clave gana.

## Dos capas

1. **Cuenta GAM** — metadatos + ruta de clave en `~/.ssh/gam_config.json`.
2. **Repo** — `attach` escribe solo config **local** de Git. Otros repos no cambian.

`switch` es una tercera capa **global**: reescribe un bloque `Host github.com` (u otro hostname) entre marcadores `# gam ACTIVE START`. Es legado. Con varios repos del mismo host, `attach` (`core.sshCommand -i clave`) o un alias `Host github-work` son más seguros.

## Alias vs mapping activo

Al crear una cuenta, GAM puede añadir:

```
Host github-work
    HostName github.com
    IdentityFile ~/.ssh/id_work_github_com
    IdentitiesOnly yes
```

El remote `git@github-work:org/repo.git` selecciona esa clave. `IdentitiesOnly` evita que ssh pruebe otras.

## Compatibilidad de config

Al arrancar, GAM lee por orden:

1. `~/.ssh/gam_config.json`
2. si no existe, `~/.ssh/ssh_manager_config.json` (y guarda una copia en el path nuevo)

Campos nuevos en `SshAccount` van con `#[serde(default)]`. Un JSON viejo **sin** `git_user_name` carga igual. No hay migración destructiva: no se tocan claves ni se reescriben repos ya atachados.

Un `attach` antiguo sigue en `.git/config` aunque actualices el binario. GAM no guarda un registro path→cuenta; `status` y `doctor` **leen** el git local del cwd para mostrártelo.

## Dónde está el código

Todo el CLI vive en `src/main.rs` (orquestación + SSH) y `src/ui.rs` (ayuda, hints, siguientes pasos). La ayuda de clap (`long_about`, `after_help`, estilos) es la asistencia en consola; esta carpeta `docs/` es la asistencia para humanos que leen el repo.

## Relacionado

- [Referencia CLI](../reference/cli.md)
- [How-to: varias cuentas](../how-to/usar-varias-cuentas.md)
