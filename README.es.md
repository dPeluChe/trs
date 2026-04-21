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

Opciones completas — binarios prebuilt, pinning de versión, directorios personalizados, troubleshooting: [`docs/support/install.md`](docs/support/install.md).

### Actualizar

```bash
trs upgrade --check    # muestra qué correría (auto-detecta canal)
trs upgrade            # actualiza binario + refresca hooks
```

Ver [`docs/features/upgrade.md`](docs/features/upgrade.md).

## Inicio rápido

```bash
# 1. Pruébalo — prefija cualquier comando con trs
trs git status
trs cargo test

# 2. Deja que tu agente lo haga automáticamente — instala hooks en
#    todos los agentes soportados (ver Agentes de IA soportados abajo)
trs init --all --global

# 3. Ve tu ahorro
trs stats                          # dashboard
trs stats --by-agent               # desglose por agente de IA
trs stats -n 30                    # límite de filas personalizado
```

Los flags funcionan en cualquier posición y stdin también es soportado:

```bash
trs git status --json              # JSON estructurado
trs --json git status              # flags en cualquier lugar
git status | trs parse git-status  # sintaxis de pipe también
```

## Agentes de IA soportados

Nueve agentes soportados end-to-end. Hook programático para Claude Code, Gemini CLI, Cursor, OpenCode, Kilo Code, Factory Droid. Solo archivo de reglas para Codex CLI, Google Antigravity, Windsurf.

| Agente | Método de install | Hook de input | Output-saver | Atribución |
|---|---|---|---|---|
| Claude Code · Gemini · Cursor | hook programático | ✓ | ✓ | `claude` / `gemini` / `cursor` |
| OpenCode · Kilo Code | plantilla de plugin | ✓ | ✓ | `opencode` / `kilo` |
| Factory Droid | hook programático | ✓ | ✓ | `claude` (comparte envelope) |
| Codex CLI · Windsurf | solo rules file | — | ✓ | `(untagged)` |
| Google Antigravity | solo rules file | — | — | `(untagged)` |

Matriz completa + caveats + rutas de config por agente: [`docs/support/agents.md`](docs/support/agents.md).

## Comandos soportados

| Categoría | Herramientas con parser dedicado |
|---|---|
| VCS | `git` (status, diff, log, branch, push, pull, fetch) |
| Rust | `cargo` (build, check, test, clippy, fmt, install) |
| JS/TS | `npm`, `pnpm`, `yarn`, `bun`, `npx`, `pnpm dlx` |
| Python | `pytest`, `pip`, `uv`, routing de `python3 -m <mod>` |
| Go | `go` (test, build, mod) |
| Tests | `pytest`, `jest`, `vitest` (parsing completo de runner) |
| Linters | `eslint`, `biome`, `ruff`, `pylint`, `golangci-lint`, `cargo clippy` |
| Files | `ls` (+ `eza`, `lsd`, `exa`), `find` (+ `fd`), `grep` (+ `rg`, `ag`, `ack`), `tree`, `tail` |
| Containers | `docker` (ps, logs, build) |
| GitHub | `gh` (pr/issue/run list + `gh api`) |
| Sistema | `ps`, `env`, `wc`, `brew`, `curl`, `wget` |

Además **chain-aware rewrite** (`cd X && cargo test`), **env-prefix preservado** (`RUSTFLAGS=x cargo build`), **sintaxis de pipe** (`cmd | trs parse …`) y `TRS_SKIP=1` para saltarse cualquier wrapping.

Referencia completa de comandos con subcommands y ejemplos: [`docs/support/commands.md`](docs/support/commands.md).

## Herramientas built-in

Comandos nativos — sin binario externo detrás.

```bash
trs json              # motor de queries jq-lite (-q '.users[].name')
trs read              # lector de archivos (-l minimal / -l aggressive)
trs search            # búsqueda de contenido basada en ripgrep
trs replace           # replace basado en ripgrep (--dry-run)
trs err               # filtro de errores (solo errores/warnings)
trs tail              # tail de logs con --errors
trs clean             # limpieza de ANSI / whitespace / dedup
trs html2md           # HTML → Markdown
trs find              # walker gitignore-aware
trs is-clean          # check de repo limpio (por exit code)
trs raw               # passthrough, sigue tracked en stats
trs stats             # dashboard de ahorro
trs debug-info        # bundle version + doctor + logs para reportar bugs
```

## Digest del proyecto

`trs ingest` recorre un repo y emite un digest Markdown compacto — estructura + archivos clave + signatures — listo para pegar al contexto de cualquier agente. Budget-aware, staleness-aware, incremental.

```bash
trs ingest                     # escribe digest, imprime path
trs ingest --budget 128k       # cabe en budget de tokens (signatures primero)
trs ingest --changed           # solo archivos no commiteados
trs ingest --since-last        # incremental desde último ingest
trs ingest --deps              # solo dependency graph
trs ingest --fresh             # reusa digest cacheado si HEAD no cambió
trs ingest --list              # digests guardados + HEAD sha + marcadores stale
```

Referencia completa: [`docs/features/ingest.md`](docs/features/ingest.md). Un ejemplo vivo — `trs ingest` corrido contra este repo — vive en [`docs/development/codebase-digest.md`](docs/development/codebase-digest.md).

## Output saver

trs comprime lo que los agentes **ven** via `trs rewrite`. `trs output-saver` cierra el gap simétrico — instala un bloque de reglas compacto en la config global de cada agente para comprimir lo que los agentes **emiten**: sin preámbulos, sin narración, resultado-primero, output estructurado cuando aplica, sin paths inventados.

```bash
trs output-saver               # scan read-only
trs output-saver --install     # instala en agentes detectados
trs output-saver --remove      # uninstall limpio
```

Ocho de nueve agentes soportados (Antigravity es solo por-proyecto). Referencia completa: [`docs/features/output-saver.md`](docs/features/output-saver.md).

## Formatos de salida

Cada comando soporta seis formatos:

```bash
trs git status                 # compact (default, humanos + agentes)
trs git status --json          # JSON estructurado
trs git status --csv           # CSV con headers
trs git status --tsv           # tab-separated
trs git status --agent         # markdown optimizado para IA
trs git status --raw           # passthrough sin procesar
```

Referencia completa con ejemplos lado-a-lado: [`docs/features/formats.md`](docs/features/formats.md).

## Seguridad y detalles finos

- **Collision check** — `trs init` detecta hooks de otras herramientas de compresión de tokens (siguiendo `@imports` también) y aborta por default. `--replace` migra limpio. Ver [`docs/support/other-token-savers.md`](docs/support/other-token-savers.md).
- **Debug bundle** — `trs debug-info` empaca version + plataforma + doctor checks + historial reciente + tee logs en un reporte listo para pegar al reportar un bug.

### Cómo se mantiene seguro

- `--no-verify` bloqueado en `git commit` / `git push` (protege pre-commit hooks de agentes).
- Comandos con `--json` / `--porcelain` pasan sin tocar.
- Si un parser falla, el output cae a truncated passthrough — nunca silencio.
- Exit codes siempre propagados del comando wrapped.
- En fallo, el output completo se guarda en `~/.trs/tee/` para recuperar.
- `trs read` nunca remueve contenido de archivos de datos JSON/YAML/TOML/XML.

## Configuración

Opcional — trs funciona sin config. Para tunear:

```toml
# ~/.trs/config.toml (o .trs/config.toml por proyecto)
[limits]
grep_max_results = 200
status_max_files = 15
passthrough_max_chars = 2000
json_max_depth = 10
```

## Por qué

<details>
<summary>La historia honesta detrás de trs.</summary>

El precio por token seguía subiendo. Cada `git status`, `cargo test` y `ls -la` que el agente volcaba a su contexto costaba dinero real, y la relación señal/ruido en esos comandos era pésima. Empezamos a escribir herramientas pequeñas — primero para nosotros, después para el equipo — que redujeran lo que el agente realmente tenía que leer.

En ese camino nos topamos con [**rtk**](https://github.com/rtk-ai/rtk) (Rust Token Killer). Para entonces nuestras herramientas venían evolucionando por su cuenta, así que enfrentamos la decisión honesta: migrar a rtk y desechar lo construido, o continuar y publicar nuestra propuesta. Decidimos continuar — más opciones en este espacio significan mejor fit para más flujos de trabajo. trs lo fuimos iterando y expandiendo conforme aprendíamos más sobre dónde se queman realmente los tokens.

Mientras más lo usábamos, más vimos que la oportunidad era más grande que los hooks de input. `trs output-saver` instala reglas en la config global de cada agente para que las respuestas también regresen más cortas. `trs audit-docs` audita CLAUDE.md / AGENTS.md buscando el bloat que cada sesión vuelve a leer. `trs ingest` comprime repositorios enteros en un índice de contexto listo para el LLM y con control de budget. Sigue siendo un binario estático único, sin deps en runtime — la historia nada más creció más allá de los hooks.

La landing page tiene el write-up completo: <https://usetrs.dev>

</details>

## Para desarrolladores

Prefiere las rutas de install prebuilt arriba a menos que estés contribuyendo. Para un checkout de fuente:

```bash
git clone https://github.com/dPeluChe/trs.git
cd trs

# Build + install en ~/.cargo/bin/
cargo install --path .

# Loop de dev
cargo test
cargo clippy -- -D warnings
cargo fmt -- --check
cargo run -- git status        # correr localmente contra el workspace
```

Ver [CONTRIBUTING.md](CONTRIBUTING.md) para guidelines, [AGENTS.md](AGENTS.md) para arquitectura, [`docs/development/`](docs/development/) para internals más profundos, y [`docs/roadmap/TASK_TODO.md`](docs/roadmap/TASK_TODO.md) para el roadmap.

## Licencia

MIT

---

<p align="center">
  Un producto de <a href="https://iteris.tech"><strong>Iteris</strong></a> · Publicado y mantenido por <a href="https://dpeluche.dev"><strong>@dPeluChe</strong></a>
</p>
