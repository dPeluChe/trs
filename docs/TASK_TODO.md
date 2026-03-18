# TARS CLI — Pending Tasks

Binary: `trs` | Language: Rust | Status: **Active development**

---

## Phase 1 — Release & Stability

### Distribution
- [ ] Push to GitHub + create first release (v0.1.0)
- [ ] npm publish (after GitHub Release with binaries)
- [ ] Homebrew formula
- [ ] Shell completions (bash, zsh, fish)

### Claude Code Integration
- [ ] Claude Code hook (auto-rewrite commands through trs, like RTK's `rtk init -g`)
- [ ] Detect pipe context — skip rewriting find/fd when piped (breaks xargs/wc/sort)

### Safety & Robustness
- [x] Passthrough inteligente — detect `--json`, `--porcelain`, `--format=json` → skip parser
- [x] Fallback seguro (3-tier): parser OK → degraded → truncated passthrough with `[trs:passthrough]` marker
- [x] Git global options support (`-C`, `--no-pager`, `--git-dir`, etc.) stripped before classification
- [x] Tee system: configurable max file size + max files via config.toml

---

## Phase 2 — Expand Parsers & Core Features

### `trs read <file>` — File reading with filter levels
- [x] Minimal filter: strip comments, normalize blank lines, preserve TODOs/pragmas
- [x] Aggressive filter: signatures-only (imports + fn/class/struct definitions, skip bodies)
- [x] Language detection by extension (Rust, Python, JS/TS, Go, Java/C/C++, Ruby, Shell)
- [x] Data file protection: JSON/YAML/TOML/XML always passthrough unmodified
- [x] Regex-based filtering (no tree-sitter dependency)
- [x] `--lines N` and `--tail N` for line range limits, `--line-numbers`

### New Parsers
- [ ] kubectl output (pods, services, deployments, logs)
- [ ] AWS CLI output (s3 ls, ec2 describe-instances, cloudwatch)
- [ ] gh pr view / gh issue view (detail view, not just list)
- [ ] cargo test (parser propio, actualmente mapeado a pytest runner)
- [ ] go test / golangci-lint / ruff check
- [ ] next build / prisma generate
- [ ] Gradle / Maven build output (skip patterns de ccp: daemon startup, deprecation warnings)

### `trs json` — Mejoras
- [ ] Ionizer sampling para arrays grandes: keep first 3 + last 3 + items con error keywords
- [ ] ID field detection heuristic (id, uuid, key, *_id suffixes)
- [ ] Reject non-JSON con hint ("use `trs parse deps` for Cargo.toml")

### Log Parser — Mejoras (LogCrunch patterns)
- [ ] Fold repetidos: runs de INFO/DEBUG >= 3 → keep first + `[...repeated N times...]` + last
- [ ] Timestamp normalization: primera ocurrencia = t0, resto = `[+Δs]` relative delta
- [ ] Stack trace detection: indented lines + "at " / "File " / "Traceback" como bloques atomicos
- [ ] Preserve siempre: ERROR, WARN, FATAL, CRITICAL, lines con "exception"/"panic"/"assert"

### Git Diff — Mejoras (DiffCrunch patterns)
- [ ] Context compression: bloques de contexto > 4 lineas → keep first 1 + `[... N unchanged ...]` + last 1
- [ ] Large diff summary: > 200 lineas → prepend file list con +adds/-dels, luego diff truncado
- [ ] Hint when truncated: `[full diff: trs git diff --raw]`

---

## Phase 3 — Analytics & Configuracion

### Configurable Limits (`~/.trs/config.toml`)
- [x] `[limits]` section: grep_max_results, grep_max_per_file, status_max_files, status_max_untracked, passthrough_max_chars, json_max_depth, json_keys_per_object, tee_max_bytes, tee_max_files
- [x] Defaults sanos, project-local override via `.trs/config.toml`
- [ ] Wire limits into existing parsers (grep, git status) — currently defined but not consumed by all handlers

### Analytics
- [ ] `trs discover` — scan Claude Code history (`~/.claude/`) for missed savings opportunities
- [ ] `trs stats --graph` — ASCII bar chart de savings ultimos 30 dias
- [ ] `trs stats --daily` — breakdown dia por dia
- [ ] Consider migrar tracker de JSONL a SQLite (WAL mode, project scope, 90-day retention)

### Diferenciacion
- [ ] Command mutation: inyectar `--porcelain` a `git status` para output mas parseable
- [ ] Streaming mode para todos los parsers (no solo tail)
- [ ] man page generation

---

## Phase 4 — Plugin System (evaluacion futura)

### TOML Filters (inspirado en tokf/RTK)
- [ ] Evaluar si hay demanda de usuarios para filtros custom
- [ ] Si se adopta: filtros como TOML con pipeline de 8 stages (strip_ansi → replace → skip/keep → dedup → head/tail → max_lines)
- [ ] Eject system: copiar filtro built-in a local para customizar
- [ ] Verify system: tests inline en cada filtro TOML
- [ ] Embedded stdlib de filtros (compilados en el binario)

### Avanzado
- [ ] SemanticDedup: shingles (3-word n-grams) + Jaccard similarity (0.80 threshold) para dedup cross-block
- [ ] Lua escape hatch para filtros complejos (sandboxed, memory/instruction limits)

---

## Learnings Reference (de competitor analysis 2026-03-18)

### Repos clonados (ghq root)
- `github.com/rtk-ai/rtk` — 10.2k stars, 64 modules, TOML filters, SQLite tracking
- `github.com/mpecan/tokf` — 126 stars, TOML pipeline, Lua, eject/verify
- `github.com/SuppieRK/ccp` — 18 stars, conservative, YAML filters, command mutation
- `github.com/open-compress/claw-compactor` — 1.6k stars, 14-stage fusion, Ionizer/LogCrunch/Neurosyntax

### Bugs criticos a evitar (de RTK P0/P1)
- Exit codes: SIEMPRE propagar el exit code del comando wrapped
- Output caps: demasiado agresivo → LLM retry loops (peor que no usar trs)
- Pipe compat: no reescribir find/fd cuando esta en pipe
- Data files: no strip comments en JSON/YAML/TOML/XML

### Ventajas competitivas de trs (mantener/expandir)
- 6 output formats (RTK solo tiene compact + JSON)
- Built-in search/replace con ripgrep
- html2md/txt2md conversiones
- Pipe syntax (`cmd | trs parse X`)
- npm distribution + cargo
- `trs json` estructura sin valores
- `trs err` filtro generico de errores
