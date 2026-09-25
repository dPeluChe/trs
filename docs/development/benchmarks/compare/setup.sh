#!/usr/bin/env bash
# Builds what compare.py runs, outside this repo and outside the clones.
#   TRS_BENCH_REPOS  clones root (default: spark's repos_root/github.com)
#   TRS_BENCH_DIR    build cache (default: ~/.cache/trs-bench)
# Clone the tools first: spark clone <owner/repo>; `spark tag list trs`.
set -euo pipefail
R=${TRS_BENCH_REPOS:-$(spark config 2>/dev/null | awk '/repos_root/{print $3}')/github.com}
C=${TRS_BENCH_DIR:-$HOME/.cache/trs-bench}
mkdir -p "$C"
echo "clones: $R"; echo "cache:  $C"

command -v rtk >/dev/null || echo "rtk not on PATH: brew install rtk (skipped otherwise)"

echo "== squeez"
CARGO_TARGET_DIR="$C/squeez-target" cargo build -q --release \
  --manifest-path "$R/claudioemmanuel/squeez/Cargo.toml"

echo "== token-saver"
[ -x "$R/ppgranger/token-saver/venv/bin/python3" ] || {
  python3 -m venv "$C/token-saver-venv"
  "$C/token-saver-venv/bin/pip" -q install -r "$R/ppgranger/token-saver/requirements.txt" 2>/dev/null || true
}

echo "== claw-compactor"
[ -x "$C/claw-venv/bin/python3" ] || python3 -m venv "$C/claw-venv"
"$C/claw-venv/bin/pip" -q install "$R/open-compress/claw-compactor"

echo "== fixtures (a cargo, bun and pytest project, 2 failing tests each)"
python3 "$(dirname "$0")/fixtures.py" "$C/fixtures"
echo done
