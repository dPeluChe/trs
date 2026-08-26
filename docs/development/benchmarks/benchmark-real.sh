#!/usr/bin/env bash
# =============================================================================
# trs benchmark-real: Real-world project benchmark
# Usage: ./scripts/benchmark-real.sh [PROJECT_PATH]
#   Default project: ~/dPeluCheData/PROJECTS/dPeluChe/_code_/labs-cameraman
#   Env: RUNS=10 ./scripts/benchmark-real.sh (default: 5 runs per test)
#
# NOTE on find: rtk silently replaces `find` with an internal walker that
# respects .gitignore (undocumented). This makes it appear faster but it
# doesn't actually execute the find command. trs executes the real command
# by default. Use `trs find --gitignore` for the equivalent fast walker.
# =============================================================================

set -e

BLUE='\033[0;34m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
RED='\033[0;31m'
CYAN='\033[0;36m'
GRAY='\033[0;90m'
BOLD='\033[1m'
NC='\033[0m'

PROJECT="${1:-$HOME/dPeluCheData/PROJECTS/dPeluChe/_code_/labs-cameraman}"

if [ ! -d "$PROJECT/.git" ]; then
    echo "Error: $PROJECT is not a git repository"
    exit 1
fi

HAS_RTK=false
command -v rtk &>/dev/null && HAS_RTK=true

RUNS=${RUNS:-5}

divider() { echo -e "\n${GRAY}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"; }
header()  { divider; echo -e "${BOLD}${BLUE}  $1${NC}"; divider; }

# =============================================================================
echo -e "${BOLD}${GREEN}"
echo "  ╔══════════════════════════════════════════════════════════════╗"
echo "  ║       trs benchmark-real: Live Project Benchmark            ║"
echo "  ╚══════════════════════════════════════════════════════════════╝"
echo -e "${NC}"
echo -e "  ${BOLD}Project:${NC} $PROJECT"
echo -e "  ${BOLD}Runs per test:${NC} $RUNS"
echo -e "  trs: $(trs --version 2>/dev/null || echo 'not found')"
[ "$HAS_RTK" = true ] && echo -e "  rtk: $(rtk --version 2>/dev/null)"
echo ""

cd "$PROJECT"
export GIT_PAGER=cat

# Detect source files extension
SRC_EXT="swift"
SRC_DIR="."
if [ -f "Cargo.toml" ]; then SRC_EXT="rs"; SRC_DIR="src"; fi
if [ -f "package.json" ]; then SRC_EXT="ts"; SRC_DIR="src"; fi
if [ -f "requirements.txt" ] || [ -f "pyproject.toml" ]; then SRC_EXT="py"; SRC_DIR="."; fi
if [ -f "go.mod" ]; then SRC_EXT="go"; SRC_DIR="."; fi

MOD_COUNT=$(git status --short 2>/dev/null | wc -l | tr -d ' ')
SRC_COUNT=$(find "$SRC_DIR" -name "*.$SRC_EXT" 2>/dev/null | wc -l | tr -d ' ')
COMMIT_COUNT=$(git log --oneline 2>/dev/null | wc -l | tr -d ' ')
BRANCH_COUNT=$(git branch -a 2>/dev/null | wc -l | tr -d ' ')
GREP_COUNT=$(grep -rn "func \|fn \|def \|function " --include="*.$SRC_EXT" "$SRC_DIR" 2>/dev/null | wc -l | tr -d ' ')

echo -e "  ${GRAY}${MOD_COUNT} modified | ${SRC_COUNT} *.$SRC_EXT files | ${GREP_COUNT} functions | ${COMMIT_COUNT} commits | ${BRANCH_COUNT} branches${NC}"
echo ""

# =============================================================================
# Run all tests via Python for precise timing
# =============================================================================

python3 -c "
import subprocess, time, os, shutil

os.chdir('$PROJECT')
os.environ['GIT_PAGER'] = 'cat'

N = $RUNS
src_ext = '$SRC_EXT'
src_dir = '$SRC_DIR'
has_rtk = shutil.which('rtk') is not None
has_tree = shutil.which('tree') is not None

# Build test list dynamically based on what's available
tests = [
    # --- Git commands ---
    ('git status',           ['git', 'status']),
    ('git diff --stat',      ['git', 'diff', '--stat']),
    ('git diff',             ['git', 'diff']),
    ('git log -20',          ['git', 'log', '-20']),
    ('git log --oneline',    ['git', 'log', '--oneline', '-50']),
    ('git log --stat -5',    ['git', 'log', '--stat', '-5']),
    ('git branch -a',        ['git', 'branch', '-a']),
    ('git diff --name-only', ['git', 'diff', '--name-only']),
    # --- File system ---
    ('ls -la',               ['ls', '-la']),
    ('ls -R (recursive)',    ['ls', '-R']),
    ('find *.' + src_ext,    ['find', src_dir, '-name', '*.' + src_ext]),
    # --- Search ---
    ('grep func defs',       ['grep', '-rn', 'func ', '--include=*.' + src_ext, src_dir]),
    # --- Environment ---
    ('env',                  ['env']),
    # --- Heavy ---
    ('git log (all)',        ['git', 'log']),
    ('git diff -U3',         ['git', 'diff', '-U3']),
]

if has_tree:
    tests.insert(11, ('tree -L 2', ['tree', '-L', '2']))

# Also test find --gitignore separately (trs-only feature)
gitignore_test = ('find --gitignore',  ['trs', 'find', '--gitignore', src_dir, '-name', '*.' + src_ext])

def run(cmd, n=N):
    times, sizes = [], []
    for _ in range(n):
        s = time.perf_counter()
        r = subprocess.run(cmd, capture_output=True)
        times.append((time.perf_counter() - s) * 1000)
        sizes.append(len(r.stdout) + len(r.stderr))
    return min(times), sum(times)/n, sizes[-1]

def safe_run(cmd, n=N):
    \"\"\"Run and detect rtk bailouts (returns 0 bytes when it refuses to execute).\"\"\"
    mn, avg, size = run(cmd, n)
    return mn, avg, size

trs_total_ms = rtk_total_ms = raw_total_ms = 0
trs_total_b = rtk_total_b = raw_total_b = 0
trs_speed_wins = rtk_speed_wins = 0
trs_size_wins = rtk_size_wins = 0
skipped = 0

hdr = f'{\"Test\":22s} {\"raw\":>7s} {\"trs\":>7s} {\"trs B\":>8s}'
if has_rtk: hdr += f' {\"rtk\":>7s} {\"rtk B\":>8s} {\"faster\":>7s} {\"smaller\":>7s}'
print(hdr)
print('-' * (95 if has_rtk else 50))

for name, cmd in tests:
    try:
        r_ms, r_avg, r_b = run(cmd)
    except FileNotFoundError:
        continue  # skip if command not found (e.g. tree)

    t_ms, t_avg, t_b = run(['trs'] + cmd)

    raw_total_ms += r_ms; raw_total_b += r_b
    trs_total_ms += t_ms; trs_total_b += t_b

    row = f'{name:22s} {r_ms:5.0f}ms {t_ms:5.0f}ms {t_b:7,}B'

    if has_rtk:
        k_ms, k_avg, k_b = safe_run(['rtk'] + cmd)

        # Detect rtk bailout: returned 0 bytes or tiny error msg when raw had real output
        if k_b < 50 and r_b > 200:
            row += f'     -- (skipped)        trs      trs'
            trs_speed_wins += 1; trs_size_wins += 1
            skipped += 1
        else:
            rtk_total_ms += k_ms; rtk_total_b += k_b
            faster = 'trs' if t_ms <= k_ms else 'rtk'
            smaller = 'trs' if t_b <= k_b else 'rtk'
            if t_ms <= k_ms: trs_speed_wins += 1
            else: rtk_speed_wins += 1
            if t_b <= k_b: trs_size_wins += 1
            else: rtk_size_wins += 1
            row += f' {k_ms:5.0f}ms {k_b:7,}B {faster:>7s} {smaller:>7s}'

    print(row)

# Special: trs find --gitignore (no rtk equivalent - rtk does it silently)
print()
gi_ms, gi_avg, gi_b = run(gitignore_test[1])
print(f'  {\"trs find --gitignore\":22s}       {gi_ms:5.0f}ms {gi_b:7,}B  (explicit fast walker)')

n_tests = len(tests)
print()
print('-' * (95 if has_rtk else 50))
print(f'{\"TOTAL\":22s} {raw_total_ms:5.0f}ms {trs_total_ms:5.0f}ms {trs_total_b:7,}B', end='')
if has_rtk:
    print(f' {rtk_total_ms:5.0f}ms {rtk_total_b:7,}B', end='')
print()

trs_pct = (raw_total_b - trs_total_b) * 100 // raw_total_b if raw_total_b > 0 else 0
print()
print(f'  trs reduction: {trs_pct}% ({raw_total_b:,} -> {trs_total_b:,} bytes)')
print(f'  tokens saved:  ~{(raw_total_b - trs_total_b) // 4:,}')
if has_rtk:
    rtk_pct = (raw_total_b - rtk_total_b) * 100 // raw_total_b if raw_total_b > 0 else 0
    print(f'  rtk reduction: {rtk_pct}% ({raw_total_b:,} -> {rtk_total_b:,} bytes)')
    if skipped > 0:
        print(f'  (rtk skipped {skipped} test(s), refused to execute command)')
    print()
    print(f'  Speed wins:       trs {trs_speed_wins}/{n_tests}  rtk {rtk_speed_wins}/{n_tests}')
    print(f'  Compression wins: trs {trs_size_wins}/{n_tests}  rtk {rtk_size_wins}/{n_tests}')
    print(f'  Total time:       trs {trs_total_ms:.0f}ms  rtk {rtk_total_ms:.0f}ms')
    print(f'  Total bytes:      trs {trs_total_b:,}B  rtk {rtk_total_b:,}B')
"

divider
echo ""
echo -e "  ${GRAY}NOTE: rtk silently replaces find/grep with internal walkers (undocumented).${NC}"
echo -e "  ${GRAY}trs always executes the real command. Use --gitignore for the fast walker.${NC}"
divider
