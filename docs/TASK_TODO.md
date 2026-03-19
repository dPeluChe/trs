# trs — Roadmap

Binary: `trs` | Language: Rust | Status: **Active development**

---

## Phase 1 — Release & Distribution

- [ ] Create first GitHub Release (v0.1.0) with precompiled binaries
- [ ] npm publish (`npm install -g tars-cli`)
- [ ] Homebrew formula
- [ ] Shell completions (bash, zsh, fish)

### AI Tool Integrations
- [ ] Claude Code hook (`trs init --claude`)
- [ ] Cursor hook (`trs init --cursor`)
- [ ] Copilot hook (`trs init --copilot`)
- [ ] Detect pipe context — skip rewriting find/fd when piped

---

## Phase 2 — New Parsers

- [ ] kubectl (pods, services, deployments, logs)
- [ ] AWS CLI (s3 ls, ec2 describe-instances, cloudwatch)
- [ ] gh pr view / gh issue view (detail view, not just list)
- [ ] go test
- [ ] next build / prisma generate
- [ ] playwright test (E2E summaries)
- [ ] Gradle / Maven build output

### Improvements to existing parsers
- [ ] Log timestamp normalization (first = t0, rest = relative delta)

---

## Phase 3 — Analytics & Configuration

- [ ] `trs discover` — scan Claude Code history for missed savings opportunities
- [ ] `trs stats --graph` — ASCII bar chart (30-day view)
- [ ] Consider migrating tracker from JSONL to SQLite (WAL mode, 90-day retention)
- [ ] Command mutation (inject `--porcelain` for more parseable output)
- [ ] Streaming mode for all parsers (not just tail)

---

## Phase 4 — Plugin System (future evaluation)

- [ ] TOML filter pipeline (inspired by tokf/RTK)
- [ ] Eject system (copy built-in filter to local for customization)
- [ ] Embedded stdlib of filters (compiled into the binary)
- [ ] SemanticDedup (shingle-based cross-block deduplication)
