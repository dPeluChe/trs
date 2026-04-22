# TASK_COMPLETED — Changelog de trabajo

Registro mensual de tareas completadas, decisiones tomadas y archivos modificados.

## Formato de archivos

Cada archivo se nombra `YYMM.md` (ej: `2603.md` = marzo 2026).

## Estructura de cada entrada

```markdown
# YYYY-MM-DD — Titulo breve de la sesion

## Context
Por que se hizo este trabajo. Contexto del problema o requerimiento.

## Completed
### Feature/Fix nombre
- Que se hizo (bullet points concretos)
- Archivos clave modificados
- Tests agregados/modificados

## Decisions
### Nombre de la decision
Que se decidio y por que. Incluir alternativas descartadas.

## Files Changed
Lista de archivos modificados con descripcion de 1 linea.
```

## Reglas

- **Un archivo por mes** — todo lo trabajado en ese mes va en el mismo archivo
- **Cada sesion es una seccion** con fecha y titulo
- **Decisions importan** — registrar lo que se descarto y por que (evita repetir analisis)
- **Files Changed** — permite rastrear que se toco sin revisar git log
- **No duplicar** lo que ya esta en git commits — enfocarse en el "por que", no en el "que"
