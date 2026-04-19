<p align="center">
  <strong>trs</strong> — compresión de terminal para agentes de IA
</p>

<p align="center">
  <a href="https://dpeluche.github.io/trs/"><strong>dpeluche.github.io/trs</strong></a> ·
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
  <a href="#instalación">Instalación</a> ·
  <a href="#qué-hace">Qué hace</a> ·
  <a href="#project-digest-trs-ingest">Digest del proyecto</a> ·
  <a href="#características">Características</a> ·
  <a href="CONTRIBUTING.md">Contribuir</a>
</p>

---

## Por qué

El precio por token seguía subiendo. Cada `git status`, `cargo test` y `ls -la` que el agente volcaba a su contexto costaba dinero real, y la relación señal/ruido en esos comandos era pésima. Empezamos a escribir herramientas pequeñas — primero para nosotros, después para el equipo — que redujeran lo que el agente realmente tenía que leer.

En ese camino nos topamos con [**rtk**](https://github.com/rtk-ai/rtk) (Rust Token Killer). Para entonces nuestras herramientas venían evolucionando por su cuenta, y el momentum de rtk confirmó lo que ya intuíamos — que el problema importaba más allá de nuestro flujo. Eso es lo que nos empujó a publicar en lugar de quedárnoslo interno. trs lo fuimos iterando y expandiendo conforme aprendíamos más sobre dónde se queman realmente los tokens.

Mientras más lo usábamos, más vimos que la oportunidad era más grande que los hooks de input. `trs output-saver` instala reglas en la config global de cada agente para que las respuestas también regresen más cortas. `trs audit-docs` audita CLAUDE.md / AGENTS.md buscando el bloat que cada sesión vuelve a leer. `trs ingest` comprime repositorios enteros en un índice de contexto listo para el LLM y con control de budget. Sigue siendo un binario estático único, sin deps en runtime — la historia nada más creció más allá de los hooks.

La landing page tiene el write-up completo: <https://dpeluche.github.io/trs/>

## Qué hace

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
cargo test: 2127 passed (71 suites, 4.9s)
# 55 KB → 58 B (99% reducción)

$ trs cargo clippy
lint: 102 issues in 39 files
src/main.rs (3):
  W unused_import 8:23
  W redundant_closure 44:30
  W dead_code 112:8
# 55 KB → 5.5 KB (90% reducción)
```

Los comandos sin parser dedicado siguen obteniendo compresión genérica (whitespace, ANSI) — ~30-40% gratis.

## Instalación

| Método | Plataforma | Notas |
|--------|------------|-------|
| **curl \| sh** | macOS / Linux | `curl -fsSL https://raw.githubusercontent.com/dPeluChe/trs/main/scripts/install.sh \| sh` — binario nativo en `~/.trs/bin/`. **Recomendado.** |
| **PowerShell** | Windows | `irm https://raw.githubusercontent.com/dPeluChe/trs/main/scripts/install.ps1 \| iex` |
| **npm** | cross-plat | `npm install -g @dpeluche/trs` — shell launcher, ~12ms de overhead. |
| **cargo** | cross-plat | `cargo install tars-cli` — compila desde fuente. Requiere Rust. |
| **Binarios** | cross-plat | [GitHub Releases](https://github.com/dPeluChe/trs/releases) — precompilados para Linux x64/arm64, macOS x64/arm64, Windows x64. |

Todos los métodos distribuyen el mismo binario nativo.

## Quick start

```bash
trs git status                     # compacto (default)
trs git status --json              # JSON estructurado
trs --json git status              # los flags funcionan en cualquier posición
git status | trs parse git-status  # sintaxis pipe también

trs init --all --global            # instala hooks en todas las herramientas detectadas
trs stats                          # dashboard de tokens ahorrados
```

## Comandos con parsers dedicados

```bash
# Git
trs git status / diff / log / branch / push / pull / fetch

# Linters (agrupados por archivo + regla)
trs cargo clippy / eslint / ruff / biome / golangci-lint

# Test runners
trs cargo test / go test / pytest / jest / vitest / npm test / pnpm test / bun test

# Archivos y búsqueda
trs ls -la / find / grep / tree

# Build y paquetes
trs cargo build / npm install / pip list

# Contenedores y GitHub CLI
trs docker ps / logs   ·   trs gh pr/issue/run list

# Sistema
trs env / wc / curl -I / wget
```

## Herramientas integradas (más que wrappers)

```bash
# Query JSON (jq-lite, sin dependencia)
curl -s api.com/users | trs json                    # estructura
curl -s api.com/users | trs json -q '.users[].name' # extrae

# Lector de archivos inteligente
trs read src/main.rs -l aggressive    # sólo signatures
trs read src/main.rs -l minimal       # sin comentarios

# Search & replace (ripgrep)
trs search src "TODO" --extension rs
trs replace src "old_fn" "new_fn" --dry-run

# Filtro de errores
trs err cargo build

# Texto
trs tail app.log --errors
trs clean --no-ansi --collapse-blanks
trs html2md https://example.com

# Fast find (walker respeta .gitignore)
trs find --gitignore . -name "*.rs"

# Utilidades
trs is-clean
trs raw gh api /repos/user/repo       # passthrough, tracked en stats
trs stats --history                   # dashboard de ahorros
```

## Project digest (`trs ingest`)

```bash
trs ingest                     # genera digest, stdout = path
trs ingest --budget 128k       # ajuste a budget de tokens (firmas primero)
trs ingest --deps              # sólo grafo de dependencias, sin contenido
trs ingest --changed           # sólo archivos con cambios sin commit
trs ingest --since-last        # diff desde el último ingest
trs ingest --fresh             # reusa digest cacheado si HEAD no cambió
trs ingest -o ~/ctx.md         # escribe a una ruta custom, sin shadow save
trs ingest --print             # contenido a stdout (default: path)
trs ingest --warn-at 40k       # warning en stderr si el digest excede N tokens
trs ingest --list              # digests guardados + HEAD sha + indicador stale
trs ingest --read miproyecto   # lee un digest guardado
```

Detección de staleness, grafos de dependencias, post-procesamiento con Ollama y truncado por budget están documentados en la [landing](https://dpeluche.github.io/trs/#digest).

## Hooks para herramientas de IA (`trs init`)

`trs init --all` instala hooks en todas las herramientas detectadas. Hooks programáticos para Claude Code, Gemini CLI, Cursor, OpenCode, Kilo, Factory Droid; archivos de rules para Codex, Google Antigravity y Windsurf. El instalador hace smart-merge en settings.json existentes — tu configuración previa se preserva.

```bash
trs init --show           # estado de todas las integraciones
trs init --all --global   # instala todo lo detectado
trs init claude           # o elige una
trs init claude --replace # migra desde un competidor (rtk, etc.)
```

Antes de escribir, `trs init` corre un chequeo de colisión: escanea
los configs target (siguiendo `@imports` en Claude/Gemini) buscando
hooks existentes de rtk o token-optimizer y aborta por default.
`--replace` limpia los hooks del competidor; `--force` instala junto
(riesgoso — doble compresión).

## Ahorro en salida (`trs output-saver`)

trs comprime lo que los agentes **ven** vía `trs rewrite`.
`trs output-saver` cierra la brecha simétrica: instala un bloque de
reglas compacto en la config global de cada agente para comprimir lo
que **emiten** — nada de preámbulos, sin narración, resultado primero,
output estructurado donde aplique, cero invención de paths.

```bash
trs output-saver            # scan read-only de los agentes detectados
trs output-saver --install  # escribe el bloque donde el scan quedó limpio
trs output-saver --print    # dump del bloque (pipe-friendly)
trs output-saver --remove   # desinstalación limpia
```

8 de 9 agentes soportados (Antigravity es per-proyecto nada más — usa
`trs init antigravity`). Claude/Gemini reciben archivo standalone más
`@import`; Cursor un `.mdc` auto-cargado; Codex/Windsurf/OpenCode/
Kilo/Droid bloque inline con sentinels HTML-comment para que reinstalar
sea idempotente.

## Formatos de salida

Cada comando soporta 6 formatos:

```bash
trs git status                # compacto (default)
trs git status --json         # JSON estructurado
trs git status --csv          # CSV con headers
trs git status --tsv          # separado por tabs
trs git status --agent        # markdown para IA
trs git status --raw          # passthrough sin procesar
```

## Características

- **30+ parsers dedicados** — git, cargo, go, npm, pnpm, docker, gh, pytest, jest, vitest, eslint, ruff, biome, golangci-lint, y más.
- **Chain-aware rewrite** — `cd X && git status` o `cargo fmt && cargo clippy` reescriben cada segmento; pipes y punto-y-comas pasan sin tocar.
- **9 integraciones de IA** — Claude, Gemini, Cursor, OpenCode, Kilo, Droid (programáticas) + Codex, Antigravity, Windsurf (rules).
- **Motor JSON query** — jq-lite incorporado, sin depender de `jq`.
- **Dashboard de ahorros** — `trs stats` muestra compresión acumulada y tokens por día.
- **Compresión genérica de fallback** — comandos sin parser igual reciben ANSI strip, whitespace collapse y dedup de líneas.

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

## Cómo se mantiene seguro

- `--no-verify` bloqueado en `git commit`/`git push` (protege pre-commit hooks de agentes).
- Comandos con `--json` / `--porcelain` pasan sin tocar.
- Si un parser falla, cae a truncated passthrough — nunca falla silencioso.
- Exit codes siempre se propagan del comando envuelto.
- En caso de error, la salida completa se guarda en `~/.trs/tee/` para recovery.
- `trs read` nunca filtra contenido de JSON/YAML/TOML/XML.

## Stack técnico

| | |
|---|---|
| Lenguaje | Rust |
| Binario | ~6 MB (LTO + strip), sin deps en runtime |
| Arranque | ~12ms en macOS / Linux |
| CLI | clap 4 (bypassed en hot path) |
| Tests | 2,154 passing, 0 warnings |
| Arquitectura | 200+ archivos, todos < ~500 LOC — [detalles](AGENTS.md) |

## Contribuir

```bash
git clone https://github.com/dPeluChe/trs.git
cd trs
cargo test                     # todos los tests deben pasar
cargo clippy -- -D warnings    # sin warnings
cargo fmt -- --check           # formato alineado
```

Ver [CONTRIBUTING.md](CONTRIBUTING.md) para guías de código, [AGENTS.md](AGENTS.md) para la arquitectura, y [docs/TASK_TODO.md](docs/TASK_TODO.md) para el roadmap.

## Licencia

MIT
