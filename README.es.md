<p align="center">
  <strong>trs</strong> — output de terminal compacto para humanos y agentes de IA
</p>

<p align="center">
  <a href="https://github.com/dPeluChe/trs/actions"><img src="https://github.com/dPeluChe/trs/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
  <a href="https://github.com/dPeluChe/trs/releases"><img src="https://img.shields.io/github/v/release/dPeluChe/trs" alt="Release"></a>
  <a href="https://www.npmjs.com/package/tars-cli"><img src="https://img.shields.io/npm/v/tars-cli" alt="npm"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License"></a>
</p>

<p align="center">
  <a href="README.md">English</a> •
  <a href="#instalar">Instalar</a> •
  <a href="#qué-hace">Qué hace</a> •
  <a href="#benchmarks">Benchmarks</a> •
  <a href="AGENTS.md">Arquitectura</a>
</p>

---

## Historia

trs nació como un proyecto de aprendizaje. Mientras exploraba cómo herramientas como [rtk](https://github.com/rtk-ai/rtk) comprimen la salida de terminal para agentes de IA, quise entender el problema a fondo — no solo usar una solución, sino construir una desde cero en Rust.

Lo que empezó como "veamos si puedo replicar esto" rápidamente se convirtió en mi herramienta diaria. El proceso de construir cada parser me enseñó qué realmente importa para reducir tokens, y en el camino trs desarrolló sus propias features: un motor de consultas JSON, un parser de linters, 6 formatos de salida, búsqueda/reemplazo integrado, y compresión genérica que funciona con cualquier comando.

Esta es la herramienta que uso todos los días con Claude Code. La comparto por si le es útil a alguien más, o como referencia para quien quiera aprender cómo funciona la compresión de output de terminal.

## Qué hace

Prefija cualquier comando con `trs` para obtener output compacto:

```bash
$ trs git status
main [ahead 1]
unstaged (3):
  M src/main.rs
  M src/lib.rs
  A src/new.rs
# 497 bytes → 81 bytes

$ trs cargo test
cargo test: ok (2012 passed, 0 failed, 70 suites, 3.21s)
# 55 KB → 58 bytes

$ trs cargo clippy
lint: 102 (102 warnings) in 39 files
src/main.rs (3):
  W unused_import 8:23
  W redundant_closure 44:30
  ...
# 55 KB → 5.5 KB
```

Los comandos sin un parser dedicado también reciben compresión básica (colapso de espacios, limpieza de ANSI) — `trs ollama list` o `trs kubectl get pods` te da ~30-40% de reducción gratis.

## Instalar

```bash
# npm (descarga binario precompilado)
npm install -g tars-cli

# Probar sin instalar
npx tars-cli git status

# Desde el código fuente
cargo install --path .

# Binarios precompilados: https://github.com/dPeluChe/trs/releases
```

## Inicio rápido

```bash
trs git status                 # output compacto
trs git status --json          # JSON estructurado
trs --json git status          # flags funcionan en cualquier posición
git status | trs parse git-status  # también funciona con pipes
```

## Comandos con parser dedicado

```bash
# Git
trs git status / diff / log / branch / push / pull / fetch

# Linters (agrupados por archivo + regla)
trs cargo clippy / eslint / ruff / biome / golangci-lint

# Test runners
trs cargo test / pytest / jest / vitest / npm test / pnpm test / bun test

# Archivos y búsqueda
trs ls -la / find / grep / tree

# Build y paquetes
trs cargo build / npm install / pip list

# Docker y GitHub CLI
trs docker ps / logs   |   trs gh pr/issue/run list

# Sistema
trs env / wc / curl -I / wget
```

## Herramientas integradas

Estas son features propias de trs que van más allá de comprimir output:

```bash
# Consulta JSON (jq-lite, sin dependencia externa)
curl -s api.com/users | trs json                    # mostrar estructura
curl -s api.com/users | trs json -q '.users[].name' # extraer valores
curl -s api.com/users | trs json -q '.meta.total'   # paths anidados

# Lector de archivos con inteligencia
trs read src/main.rs -l aggressive    # solo firmas (93% reducción)
trs read src/main.rs -l minimal       # sin comentarios, código limpio

# Búsqueda y reemplazo (con ripgrep)
trs search src "TODO" --extension rs
trs replace src "old_fn" "new_fn" --dry-run

# Filtro de errores (funciona con cualquier comando)
trs err cargo build

# Utilidades
trs tail app.log --errors             # solo líneas de error
trs clean --no-ansi --collapse-blanks # limpiar texto
trs html2md https://example.com       # HTML → Markdown
trs is-clean                          # estado git: exit 0=limpio, 1=sucio
trs raw gh api /repos/user/repo       # sin compresión, registrado en stats
trs stats --history                   # dashboard de ahorro de tokens
```

## Formatos de salida

Cada comando soporta 6 formatos:

```bash
trs git status                # compact (default)
trs git status --json         # JSON estructurado
trs git status --csv          # CSV con headers
trs git status --tsv          # separado por tabs
trs git status --agent        # optimizado para IA
trs git status --raw          # sin procesar
```

## Benchmarks

vs [rtk](https://github.com/rtk-ai/rtk) (el proyecto que inspiró este):

**Score: trs 13 wins, rtk 4 wins, 1 empate** en 18 tests.

| Comando | Raw | trs | rtk | Ganador |
|---------|-----|-----|-----|---------|
| `cargo test` | 55 KB | 58 B | 62 B | trs |
| `cargo clippy` | 55 KB | 5.5 KB | — | trs |
| `git status` | 13.6 KB | 876 B | 343 B | rtk |
| `ls -la` | 1.4 KB | 227 B | 291 B | trs |
| `env` | 3.0 KB | 807 B | 1.1 KB | trs |
| `find *.rs` | 3.9 KB | 2.1 KB | 3.9 KB | trs |

**Velocidad**: trs agrega ~3ms de overhead, rtk ~7ms.

Ejecútalo tú mismo: `./scripts/benchmark.sh`

## Contribuir

Ver [CONTRIBUTING.md](CONTRIBUTING.md) para guías de código, cómo agregar parsers, y cómo proponer nuevos comandos.

```bash
cargo test                     # 2,039 tests deben pasar
cargo clippy -- -D warnings    # sin warnings
cargo fmt -- --check           # formato correcto
```

## Agradecimientos

- [rtk](https://github.com/rtk-ai/rtk) — el proyecto que inició todo esto. Su enfoque de reducción de tokens para agentes de IA me mostró que el problema valía la pena, y estudiar su código me enseñó mucho sobre diseño de CLIs en Rust.
- [claw-compactor](https://github.com/open-compress/claw-compactor) — patrones de compresión (LogCrunch, DiffCrunch, Ionizer).
- [tokf](https://github.com/mpecan/tokf) — concepto de pipeline de filtros TOML.

## Licencia

MIT
