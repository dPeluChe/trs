# TASK_COMPLETED: changelog de trabajo

Registro mensual de tareas completadas, decisiones tomadas y archivos modificados.

## Formato de archivos

Cada archivo se nombra `YYMM.md` (ej: `2603.md` = marzo 2026).

## Estructura de cada entrada

```markdown
# YYYY-MM-DD: titulo breve de la sesion

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

- **Un archivo por mes**: todo lo trabajado en ese mes va en el mismo archivo
- **Cada sesion es una seccion** con fecha y titulo
- **Decisions importan**: registrar lo que se descarto y por que (evita repetir analisis)
- **Files Changed**: permite rastrear que se toco sin revisar git log
- **No duplicar** lo que ya esta en git commits, enfocarse en el "por que", no en el "que"

## Puntuacion

Las entradas nuevas no llevan em dash (`—`). Es la marca mas fuerte de texto
generado, y la regla que trs inyecta en la config de cada agente la prohibe,
asi que el repo la cumple tambien. Usa dos puntos para una etiqueta seguida de
su explicacion, coma para un inciso, punto para dos frases independientes.

Los archivos `YYMM.md` ya escritos conservan su puntuacion original: son un
registro fechado, y reescribirlo lo falsea.
