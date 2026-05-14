#!/usr/bin/env bash
# =============================================================================
# trs chain-rewrite benchmark
# Measures how well the generalized chain rewriter captures commands that
# AI agents run as chains (A && B) vs. the old single-command-only behavior.
#
# Each test case feeds a command through `trs rewrite` as if it came from the
# Claude Code PreToolUse hook. Output shows whether each segment was rewritten.
#
# Usage:
#   ./docs/development/benchmarks/chain-rewrite.sh
# =============================================================================

set -e

GREEN='\033[0;32m'
YELLOW='\033[0;33m'
RED='\033[0;31m'
CYAN='\033[0;36m'
GRAY='\033[0;90m'
BOLD='\033[1m'
NC='\033[0m'

# Prefer local build if available (tests new changes before install)
if [ -z "${TRS_BIN:-}" ]; then
    REPO_ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
    if [ -x "$REPO_ROOT/target/release/trs" ]; then
        TRS_BIN="$REPO_ROOT/target/release/trs"
    elif [ -x "$REPO_ROOT/target/debug/trs" ]; then
        TRS_BIN="$REPO_ROOT/target/debug/trs"
    else
        TRS_BIN="trs"
    fi
fi

if ! command -v "$TRS_BIN" &>/dev/null && [ ! -x "$TRS_BIN" ]; then
    echo "error: '$TRS_BIN' not found" >&2
    exit 1
fi

printf "${GRAY}using: %s${NC}\n\n" "$TRS_BIN"

# Each case: <description>¦<input command>¦<expected outcome>
# Using ¦ (U+00A6) as separator to avoid collisions with shell pipes in inputs.
# Expected: "rewritten" if we want it rewritten, "passthrough" if not
CASES=(
    "single command¦git status¦rewritten"
    "simple chain¦cargo fmt && cargo clippy¦rewritten"
    "cd chain¦cd /tmp && git status¦rewritten"
    "triple chain¦cd /tmp && git status && git log¦rewritten"
    "mixed chain¦cd /tmp && echo hello¦passthrough"
    "chain with pipe¦cd /tmp && git status | less¦passthrough"
    # Pipes/redirects rewrite the FIRST segment only (trs rewrites the
    # data-producing command, leaves head/grep/file untouched).
    "pure pipe¦git log | grep fix¦rewritten"
    "semicolon chain¦git status ; git log¦passthrough"
    "redirection¦git diff > out.txt¦rewritten"
    "already trs¦trs git status¦passthrough"
    "env assignment¦FOO=bar¦passthrough"
    # Transparent prefixes: wrapper stays in front; trs slots in front of the inner cmd.
    "wrapper time¦time cargo test¦rewritten"
    "wrapper nohup¦nohup cargo build¦rewritten"
    "wrapper noglob¦noglob git status¦rewritten"
    "wrapper command¦command git log -5¦rewritten"
    "wrapper exec¦exec git status¦rewritten"
    "wrapper builtin¦builtin git status¦rewritten"
    "wrapper nocorrect¦nocorrect git status¦rewritten"
    "wrapper setsid¦setsid cargo build¦rewritten"
    "wrapper unbuffer¦unbuffer cargo test¦rewritten"
    "wrapper stdbuf¦stdbuf cargo test¦rewritten"
    "wrapper + env¦RUSTFLAGS=-C time cargo test¦rewritten"
    "nested wrappers¦nohup time cargo build¦rewritten"
    "wrapper + chain¦time cargo fmt && cargo clippy¦rewritten"
    "wrapper false-prefix (timeout)¦timeout 5 git status¦rewritten"
    "wrapper + skip inner (cd)¦time cd /tmp¦passthrough"
    # Venv runners — v0.6.2 transparent_prefix additions. Inner cmd goes
    # through trs's parsers (pytest / python / etc.) instead of bypassing.
    "wrapper poetry run¦poetry run pytest¦rewritten"
    "wrapper poetry run with args¦poetry run python manage.py migrate¦rewritten"
    "wrapper uv run¦uv run pytest -v¦rewritten"
    "wrapper pdm run¦pdm run pytest¦rewritten"
    # Inline scripts and generic CLIs added to REWRITE_PREFIXES in v0.6.2.
    # These get generic ANSI/whitespace compression even without a parser.
    "bash inline¦bash -c \"git status\"¦rewritten"
    "node inline¦node -e \"console.log(1)\"¦rewritten"
    "awk filter¦awk /pattern/ file.txt¦rewritten"
    "du size¦du -h dist/¦rewritten"
    "jq query¦jq -r .name package.json¦rewritten"
    # Bypass channels — v0.6.2 adds TRS_DISABLE alias + env-wrapped form.
    "bypass trs_skip¦TRS_SKIP=1 git status¦passthrough"
    "bypass trs_disable¦TRS_DISABLE=1 npx tsc¦passthrough"
    "bypass env-wrapped¦env TRS_DISABLE=1 npx tsc¦passthrough"
    # Wrappers we deliberately do NOT strip (next token is script name).
    "bun run stays wrapped¦bun run typecheck¦rewritten"
    "pnpm run stays wrapped¦pnpm run build¦rewritten"
)

pass=0
fail=0
total=0

printf "${BOLD}trs chain-rewrite benchmark${NC}\n"
printf "${GRAY}────────────────────────────────────────────────${NC}\n\n"

for case in "${CASES[@]}"; do
    IFS='¦' read -r desc input expected <<< "$case"
    total=$((total + 1))

    # Wrap as Claude Code hook JSON input
    input_json=$(printf '{"tool_name":"Bash","tool_input":{"command":"%s"}}' "$input")

    output=$(printf '%s' "$input_json" | "$TRS_BIN" rewrite 2>/dev/null)

    if [ -n "$output" ]; then
        actual="rewritten"
        rewritten_cmd=$(printf '%s' "$output" | grep -o '"command":"[^"]*"' | sed 's/"command":"//;s/"$//' || echo "")
    else
        actual="passthrough"
        rewritten_cmd=""
    fi

    if [ "$actual" = "$expected" ]; then
        printf "${GREEN}✓${NC} %-25s ${GRAY}%-45s${NC}" "$desc" "$input"
        if [ -n "$rewritten_cmd" ]; then
            printf " ${CYAN}→${NC} %s\n" "$rewritten_cmd"
        else
            printf "\n"
        fi
        pass=$((pass + 1))
    else
        printf "${RED}✗${NC} %-25s ${GRAY}%-45s${NC}\n" "$desc" "$input"
        printf "    expected=${YELLOW}%s${NC}  got=${RED}%s${NC}\n" "$expected" "$actual"
        fail=$((fail + 1))
    fi
done

printf "\n${GRAY}────────────────────────────────────────────────${NC}\n"
if [ $fail -eq 0 ]; then
    printf "${GREEN}%d/%d passed${NC}\n" "$pass" "$total"
    exit 0
else
    printf "${RED}%d/%d failed${NC}\n" "$fail" "$total"
    exit 1
fi
