<p align="center">
  <strong>trs</strong> — <strong>T</strong>oken-<strong>R</strong>educing <strong>S</strong>hell · compresión de salida terminal para agentes de IA
</p>

<p align="center">
  <a href="https://usetrs.dev"><strong>usetrs.dev</strong></a> ·
  <a href="https://github.com/dPeluChe/trs">GitHub</a> ·
  <a href="https://www.npmjs.com/package/@dpeluche/trs">npm</a> ·
  <a href="README.md">English</a>
</p>

<p align="center">
  <a href="https://github.com/dPeluChe/trs/actions"><img src="https://github.com/dPeluChe/trs/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
  <a href="https://github.com/dPeluChe/trs/releases"><img src="https://img.shields.io/github/v/release/dPeluChe/trs" alt="Release"></a>
  <a href="https://www.npmjs.com/package/@dpeluche/trs"><img src="https://img.shields.io/npm/v/@dpeluche/trs" alt="npm"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License"></a>
</p>

<p align="center">
  <a href="#qué-es-trs">Qué</a> ·
  <a href="#instalación">Instalar</a> ·
  <a href="#inicio-rápido">Inicio rápido</a> ·
  <a href="#agentes-de-ia-soportados">Agentes</a> ·
  <a href="#comandos-soportados">Comandos</a> ·
  <a href="#herramientas-built-in">Built-in</a> ·
  <a href="#digest-del-proyecto">Digest</a> ·
  <a href="#por-qué">Por qué</a>
</p>

---

## Qué es trs

Prefija cualquier comando con `trs` (o deja que `trs init` lo conecte a tu herramienta de IA). El binario ejecuta tu comando, parsea la salida, y emite una versión compacta pensada para humanos y LLMs.

```bash
$ trs git status
main [ahead 1]
unstaged (3):
  M src/main.rs
  M src/lib.rs
  A src/new.rs
# 1.4 KB → 336 B (76% reducción)

$ trs cargo test
cargo test: 2186 passed (71 suites, 4.9s)
# 55 KB → 58 B (99% reducción)

$ trs cargo clippy
lint: 102 issues in 39 files
src/main.rs (3):
  W unused_import 8:23
  W redundant_closure 44:30
  W dead_code 112:8
# 55 KB → 5.5 KB (90% reducción)
```

Los comandos sin parser dedicado siguen obteniendo compresión genérica (whitespace, ANSI) — ~30–40% gratis.

## Instalación

Binario nativo único — **macOS (arm64/x64), Linux (arm64/x64), Windows (x64)**.

```bash
# macOS / Linux
curl -fsSL https://usetrs.dev/install.sh | sh

# npm (todas las plataformas)
npm install -g @dpeluche/trs

# cargo (compila desde fuente)
cargo install trs-cli

# Windows (PowerShell)
irm https://usetrs.dev/install.ps1 | iex
```

[Guía completa de instalación — binarios prebuilt, fijar versión, directorios personalizados, solución de problemas →](docs/support/install.md)

### Actualizar

```bash
trs upgrade --check    # muestra qué correría (auto-detecta el canal)
trs upgrade            # actualiza el binario + refresca hooks
trs doctor             # verifica que la instalación esté sana
```

Referencias: [actualizar](docs/features/upgrade.md) · [doctor](docs/features/doctor.md).

## Inicio rápido

```bash
# 1. Instala hooks en cada agente detectado (el camino principal)
trs init --all --global
trs init --all --global --dry-run  # preview sin escribir

# 2. Ve tu ahorro
trs stats                          # dashboard
trs stats --by-agent               # desglose por agente de IA
trs stats -n 30                    # límite de filas personalizado

# 3. Desinstala cuando quieras — flujo simétrico
trs uninstall                      # interactivo
trs uninstall --all --yes          # scriptable
```

[Referencia completa de `trs uninstall` →](docs/features/uninstall.md)

## Agentes de IA soportados

Doce agentes cubiertos de extremo a extremo. Hook programático para Claude Code, Gemini CLI, Cursor, OpenCode, Kilo Code, Pi Coding Agent, Factory Droid, VS Code Copilot, Antigravity IDE y Antigravity CLI. Solo archivo de reglas para Codex CLI y Devin Desktop (ex-Windsurf).

| Agente | Método de instalación | Hook de entrada | Output-saver | Etiqueta en stats |
|---|---|---|---|---|
| Claude Code · Gemini · Cursor | hook programático | ✓ | ✓ | `claude` / `gemini` / `cursor` |
| OpenCode · Kilo Code | plantilla de plugin | ✓ | ✓ | `opencode` / `kilo` |
| Pi Coding Agent | hook programático (extensión) | ✓ | — | `pi` |
| Factory Droid | hook programático | ✓ | ✓ | `claude` (mismo envelope) |
| VS Code Copilot | hook programático | ✓ | — | `vscode` |
| Antigravity IDE · Antigravity CLI (`agy`) | archivo de reglas ([notas](docs/development/antigravity-hooks-research.md)) | — | ✓ | `(untagged)` |
| Codex CLI · Devin Desktop (ex-Windsurf) | solo archivo de reglas | — | ✓ | `(untagged)` |

[Matriz completa, detalles y rutas de configuración por agente →](docs/support/agents.md)

### Uso standalone (opcional)

También puedes invocar `trs` directamente sin hooks — útil para scripts, CI, o para probarlo antes de usar `trs init`:

```bash
trs git status
trs cargo test
trs git status --json              # JSON estructurado
trs --json git status              # los flags funcionan en cualquier posición
git status | trs parse git-status  # también soporta pipe
```

## Comandos soportados

| Categoría | Herramientas con parser dedicado |
|---|---|
| VCS | `git` (status, diff, log, branch, push, pull, fetch, show, stash, grep) |
| Rust | `cargo` (build, check, test, clippy, fmt, install) |
| JS/TS | `npm`, `pnpm`, `yarn`, `bun`, `npx`, `pnpm dlx` |
| Python | `pytest`, `pip`, `uv`, dispatch de `python3 -m <mod>` |
| Go | `go` (test, build, mod) |
| Tests | `pytest`, `jest`, `vitest` (parseo completo del runner) |
| Linters | `eslint`, `biome`, `ruff`, `pylint`, `golangci-lint`, `cargo clippy`, `tsc` |
| Archivos | `ls` (+ `eza`, `lsd`, `exa`), `find` (+ `fd`), `grep` (+ `rg`, `ag`, `ack`), `tree`, `tail`, `cat`, `head`, `sed -n X,Yp` |
| Containers | `docker` (ps, logs, build) |
| GitHub | `gh` (pr list, pr view, issue list, run list + `gh api`) |
| Sistema | `ps`, `env`, `wc`, `brew`, `curl`, `wget` |

También incluye **chain-aware rewrite** (`cd X && cargo test`), **preservación de env-prefix** (`RUSTFLAGS=x cargo build`) y **sintaxis de pipe** (`cmd | trs parse …`).

[Referencia completa de comandos con subcomandos y ejemplos →](docs/support/commands.md)

## Herramientas built-in

Comandos nativos — sin binario externo detrás.

```bash
trs json              # motor de queries tipo jq-lite (-q '.users[].name')
trs read              # lector de archivos (-l minimal / -l aggressive)
trs search            # búsqueda de contenido basada en ripgrep
trs replace           # replace basado en ripgrep (--dry-run)
trs err               # filtro de errores (solo errores/warnings)
trs tail              # tail de logs con --errors
trs clean             # limpieza de ANSI / whitespace / dedup
trs html2md           # HTML → Markdown
trs find              # walker que respeta .gitignore
trs is-clean          # verifica si el repo está limpio (por exit code)
trs raw               # passthrough, pero sigue registrándose en stats
trs stats             # dashboard de ahorro
trs debug-info        # empaqueta version + doctor + logs para reportar bugs
```

## Digest del proyecto

`trs ingest` recorre un repo y emite un digest Markdown compacto — estructura + archivos clave + firmas de funciones — listo para pegar en el contexto de cualquier agente. Respeta un budget de tokens, detecta digests obsoletos (stale) y soporta generación incremental.

```bash
trs ingest                     # escribe el digest, imprime el path
trs ingest --budget 128k       # ajusta al budget de tokens (firmas primero)
trs ingest --changed           # solo archivos con cambios sin commitear
trs ingest --since-last        # incremental desde el último ingest
trs ingest --deps              # solo el grafo de dependencias
trs ingest --fresh             # reusa el digest en caché si HEAD no cambió
trs ingest --list              # digests guardados + HEAD sha + si están stale
```

[Referencia completa de `trs ingest` →](docs/features/ingest.md) · [Ejemplo vivo — trs aplicado a sí mismo →](docs/development/codebase-digest.md)

## Output saver

`trs` comprime lo que los agentes **ven** (via `trs rewrite`). `trs output-saver` cierra el otro lado del bucle — instala un bloque de reglas compacto en la configuración global de cada agente para comprimir lo que los agentes **emiten**: sin preámbulos, sin narración, resultado primero, output estructurado cuando aplica, y sin paths inventados.

```bash
trs output-saver               # escaneo de solo lectura
trs output-saver --install     # instala en los agentes detectados
trs uninstall --output-saver   # quita el bloque de todos los agentes
```

Los diez agentes soportados (Antigravity IDE + CLI comparten el `~/.gemini/GEMINI.md` de Gemini). [Referencia completa de `trs output-saver` →](docs/features/output-saver.md)

## Formatos de salida

Cada comando soporta seis formatos:

```bash
trs git status                 # compact (por defecto, humanos + agentes)
trs git status --json          # JSON estructurado
trs git status --csv           # CSV con headers
trs git status --tsv           # separado por tabs
trs git status --agent         # Markdown optimizado para IA
trs git status --raw           # passthrough sin procesar
```

[Referencia completa de formatos con ejemplos lado-a-lado →](docs/features/formats.md)

## Por qué

<details>
<summary>La historia honesta detrás de trs.</summary>

El precio por token seguía subiendo. Cada `git status`, `cargo test` y `ls -la` que el agente volcaba a su contexto costaba dinero real, y la relación señal/ruido en esos comandos era pésima. Empezamos a escribir herramientas pequeñas dentro del equipo de Iteris para reducir lo que el agente realmente tenía que leer.

En ese camino nos topamos con [**rtk**](https://github.com/rtk-ai/rtk) (Rust Token Killer). Para entonces nuestras herramientas venían evolucionando por su cuenta, así que enfrentamos la decisión honesta: migrar a rtk y desechar lo construido, o continuar y publicar nuestra propuesta. Decidimos continuar — más opciones en este espacio significan un mejor encaje con más flujos de trabajo. Seguimos iterando y expandiendo `trs` conforme aprendíamos dónde se queman realmente los tokens.

Mientras más lo usábamos, más vimos que la oportunidad era más grande que solo los hooks de entrada. La historia se convirtió en cuatro herramientas complementarias:

- [`trs rewrite`](docs/features/init.md) — compresión de la entrada en cada tool-call del agente.
- [`trs output-saver`](docs/features/output-saver.md) — bloque de reglas que acorta las respuestas de vuelta.
- [`trs audit-docs`](docs/features/audit-docs.md) — encuentra contenido redundante, duplicados y `@imports` muertos en los archivos de instrucciones que cada sesión re-carga.
- [`trs ingest`](docs/features/ingest.md) — digest de todo un repo, listo para LLM y con control de budget.

</details>

## Desde código fuente

Prefiere las rutas de instalación prebuilt de arriba a menos que estés contribuyendo. Para un checkout desde fuente:

```bash
git clone https://github.com/dPeluChe/trs.git
cd trs

# Build + install en ~/.cargo/bin/
cargo install --path .

# Loop de desarrollo
cargo test
cargo clippy -- -D warnings
cargo fmt -- --check
cargo run -- git status        # corre localmente contra el workspace
```

## For contributors

Mantenemos la referencia de contribución en inglés — los términos técnicos (wire formats, benchmarks, PR checklist, etc.) no tienen equivalente limpio en español.

| Link | Topic |
|---|---|
| **[Contributing guide →](CONTRIBUTING.md)** | Code style, review process, PR checklist |
| **[Architecture overview →](AGENTS.md)** | File map and module responsibilities |
| **[Development internals →](docs/development/)** | Agent wire formats, benchmarks, safety guarantees |
| **[Roadmap →](docs/roadmap/TASK_TODO.md)** | Active items and planned work |
| **[Codebase digest →](docs/development/codebase-digest.md)** | Auto-generated project map for agents |

## Licencia

MIT

---

<p align="center">
  Un producto de <a href="https://iteris.tech"><strong>Iteris</strong></a> · Publicado y mantenido por <a href="https://dpeluche.dev"><strong>@dPeluChe</strong></a>
</p>
