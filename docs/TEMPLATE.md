# Plantilla de documentación

Copia este archivo a la carpeta Diátaxis que corresponda y rellena cada sección. No dejes encabezados vacíos: bórralos si no aplican.

**Tipo** (elige uno):

- Tutorial — el lector no conoce el tema; un camino feliz, con resultado visible.
- How-to — el lector ya usa GAM; quiere completar una tarea.
- Referencia — hechos: flags, archivos, códigos de salida. Sin narración.
- Explicación — contexto y decisiones. No es una receta.

```md
# <título en una línea>

<!--
audience: (quién lo lee: usuario de GAM / contribuidor / agente)
type: tutorial | how-to | reference | explanation
-->

## Qué vas a conseguir

Una frase. Si no puedes escribirla, el documento no tiene foco.

## Cuándo usarlo

Lista corta de situaciones. Enlaza a otras páginas si esta no es la adecuada.

## Requisitos

- Binario `gam-cli` en PATH
- …

## Pasos

1. …
2. …

Muestra comandos reales. El output de ejemplo debe coincidir con el CLI actual (`-v`, `doctor`, etc.).

## Resultado esperado

Qué debe verse o quedar en disco (`~/.ssh/gam_config.json`, `.git/config`, `~/.ssh/config`).

## Si algo falla

Síntoma → causa probable → comando (`gam-cli doctor`, `gam-cli list -v`).

## Relacionado

- [otra página](../…)
```

## Convenciones

- Un tema por archivo. Nombre: `kebab-case.md`.
- Comandos en `gam-cli …`. El menú interactivo es `gam-cli` sin argumentos.
- No documentes claves privadas ni pegues `gam_config.json` reales.
- Idioma: español, mismos nombres de flags que el CLI (inglés: `--verbose`).
- Tras cambiar un comando, actualiza `docs/reference/cli.md` en el mismo PR.
