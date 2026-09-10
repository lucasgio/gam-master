# How-to: adjuntar un repositorio a una cuenta

<!--
audience: desarrollador dentro de un clone git
type: how-to
-->

## Qué vas a conseguir

El repo actual usa `user.name` / `user.email` y la clave SSH de una cuenta GAM.

## Requisitos

- Estar dentro de un work tree git
- Al menos una cuenta (`gam-cli list`)

## Pasos

```bash
cd /ruta/al/repo
gam-cli attach
```

Elige la cuenta en el selector. El CLI muestra origin e identidad actual **antes** de escribir, y los valores aplicados **después**.

## Resultado esperado

En `.git/config`:

```
[user]
    name = …
    email = …
[core]
    sshCommand = ssh -i /Users/…/.ssh/id_… -o IdentitiesOnly=yes
```

`gam-cli status` debe listar esos valores bajo **Current directory**.

## Si algo falla

| Síntoma | Qué hacer |
|---|---|
| `not a git repository` | `cd` a la raíz del clone (o cualquier subcarpeta del work tree). |
| Solo se setea SSH | La cuenta no tiene `git_user_name` / `git_user_email`. Vuelve a `gam-cli add` o edita `~/.ssh/gam_config.json`. |
| Key MISSING | `gam-cli doctor` — el `IdentityFile` no está en disco. |

## Relacionado

- [Varias cuentas](usar-varias-cuentas.md)
- [Referencia CLI](../reference/cli.md)
