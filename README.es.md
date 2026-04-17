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

trs nació como un proyecto de aprendizaje. Las sesiones de IA con código quemaban decenas de miles de tokens solo renderizando `git status`, `cargo test` y `ls -la` al contexto del agente — y teníamos la convicción de que existía un mejor ratio señal/ruido. Estudiamos los trabajos previos en el espacio y escribimos trs desde cero en Rust para ajustarlo a nuestro flujo: un binario estático único, sin dependencias en runtime, parsers propios por comando, y una historia de ingest afinada para context windows de LLM.

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
```

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
| Tests | 2,127 passing, 0 warnings |
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
