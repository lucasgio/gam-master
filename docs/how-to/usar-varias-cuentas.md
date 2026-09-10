# How-to: usar varias cuentas Git en la misma máquina

<!--
audience: desarrollador con cuentas work y personal
type: how-to
-->

## Qué vas a conseguir

Commits y `git fetch`/`push` con la identidad correcta en cada repositorio, sin pisar la otra cuenta.

## Cuándo usarlo

Tienes (o vas a tener) más de un usuario en GitHub/GitLab y un solo `~/.ssh`.

## Pasos

1. Crea una cuenta por identidad:

   ```bash
   gam-cli add    # p. ej. work
   gam-cli add    # p. ej. personal
   gam-cli list -v
   ```

2. Añade cada clave pública en el host correspondiente.

3. En **cada repo**, elige la cuenta:

   ```bash
   cd ~/src/proyecto-trabajo
   gam-cli attach
   ```

   Eso escribe `user.name`, `user.email` y `core.sshCommand` en `.git/config` (solo ese repo).

4. (Opcional) Usa el alias SSH en el remote para no depender solo de `core.sshCommand`:

   ```bash
   git remote set-url origin git@github-work:org/repo.git
   ```

   El alias aparece en `gam-cli list` como **SSH alias**.

5. Comprueba:

   ```bash
   gam-cli status -v
   git fetch
   ```

## Qué no hacer

`gam-cli switch` cambia el bloque **global** `Host github.com` en `~/.ssh/config`. Sirve como legado; con varios repos del mismo host, `attach` por repo es el flujo correcto.

## Si algo falla

- Commit con el email incorrecto → no corriste `attach` en ese repo. `gam-cli status` muestra `user.email` local.
- `Permission denied` → la clave no está en el host, o `attach` apunta a otro `IdentityFile`. `gam-cli doctor -v`.
- Dos cuentas, un solo `Host github.com` → usa alias `git@github-<cuenta>:…`.

## Relacionado

- [Attach](adjuntar-un-repo.md)
- [Cómo funciona](../explanation/como-funciona.md)
