#!/usr/bin/env bash
# =============================================================================
# trs benchmark — Compare raw, trs, and rtk output
# Usage: ./scripts/benchmark.sh [--all]
#   Without --all: pauses between each test (press Enter to continue)
#   With --all: runs all tests without pausing
# =============================================================================

set -e

# Always run from project root regardless of how the script was invoked
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"
cd "$PROJECT_ROOT"

BLUE='\033[0;34m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
RED='\033[0;31m'
CYAN='\033[0;36m'
GRAY='\033[0;90m'
BOLD='\033[1m'
NC='\033[0m'

RUN_ALL=false
if [ "$1" = "--all" ]; then RUN_ALL=true; fi

HAS_RTK=false
if command -v rtk &>/dev/null; then HAS_RTK=true; fi

HAS_TS=false
TS_WRAP_CMD=""
# Look for token-saver in common locations
for ts_dir in \
    "$HOME/dPeluCheData/PROJECTS/dPeluChe/_code_/_repos_2_learn/github.com/ppgranger/token-saver" \
    "$HOME/token-saver" \
    "$HOME/.token-saver"; do
    if [ -f "$ts_dir/scripts/wrap.py" ] && [ -d "$ts_dir/venv" ]; then
        TS_WRAP_CMD="$ts_dir/venv/bin/python3 $ts_dir/scripts/wrap.py"
        HAS_TS=true
        break
    fi
done

# Score tracking
TRS_WINS=0
RTK_WINS=0
TS_WINS=0
TIES=0
TOTAL=0

# Cumulative timing (milliseconds) — local tests only (deterministic)
RAW_LOCAL_MS=0
TRS_LOCAL_MS=0
RTK_LOCAL_MS=0
TS_LOCAL_MS=0
LOCAL_TESTS=0

# Network test timing (high variance — shown separately)
RAW_NET_MS=0
TRS_NET_MS=0
RTK_NET_MS=0
TS_NET_MS=0
NET_TESTS=0

# trs-only timing (separate so it doesn't inflate comparisons)
TRS_ONLY_MS=0
TRS_ONLY_COUNT=0

# Speed wins tracking
TRS_SPEED_WINS=0
RTK_SPEED_WINS=0
TS_SPEED_WINS=0
SPEED_TIES=0

# Portable ms timer (macOS + Linux)
if command -v gdate &>/dev/null; then
    _date_cmd="gdate"  # macOS with coreutils
elif date +%s%3N &>/dev/null 2>&1 && [ "$(date +%s%3N | wc -c | tr -d ' ')" -gt 12 ]; then
    _date_cmd="date"   # Linux
else
    _date_cmd=""        # fallback: no ms timing
fi

now_ms() {
    if [ -n "$_date_cmd" ]; then
        $_date_cmd +%s%3N
    else
        # fallback using python (available on macOS)
        python3 -c 'import time; print(int(time.time()*1000))'
    fi
}

# Run a command and capture output + elapsed ms
# Usage: timed_run VARNAME TIMEVAR cmd [args...]
#   Sets $VARNAME to stdout and $TIMEVAR to elapsed ms
timed_run() {
    local _var="$1" _tvar="$2"; shift 2
    local _start _end
    _start=$(now_ms)
    local _out _exit=0
    _out=$("$@" 2>/dev/null) || _exit=$?
    if [ "$_exit" -eq 124 ]; then
        _out="(timeout)"
    elif [ "$_exit" -ne 0 ] && [ -z "$_out" ]; then
        _out="(error $_exit)"
    fi
    _end=$(now_ms)
    eval "$_var=\$_out"
    eval "$_tvar=$((_end - _start))"
}

divider() { echo -e "\n${GRAY}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"; }
header()  { divider; echo -e "${BOLD}${BLUE}  $1${NC}"; divider; }
label()   { echo -e "  ${CYAN}[$1]${NC} ${GRAY}($2 bytes)${NC}"; }

bytes() { echo -n "$1" | wc -c | tr -d ' '; }

pause() {
    if [ "$RUN_ALL" = false ]; then
        echo -e "\n  ${GRAY}Press Enter for next test (or q to quit)...${NC}"
        read -r input
        if [ "$input" = "q" ]; then print_summary; exit 0; fi
    fi
}

time_label() {
    local ms="$1"
    if [ "$ms" -lt 1000 ]; then
        echo "${ms}ms"
    else
        local sec=$((ms / 1000)) rem=$((ms % 1000 / 100))
        echo "${sec}.${rem}s"
    fi
}

speed_badge() {
    # speed_badge <tool_ms> <raw_ms>  — shows overhead vs raw
    local tool_ms="$1" raw_ms="$2"
    if [ "$raw_ms" -gt 0 ] && [ "$tool_ms" -gt 0 ]; then
        local overhead=$(( (tool_ms - raw_ms) * 100 / raw_ms ))
        if [ "$overhead" -le 10 ]; then
            echo -e "${GREEN}⚡ +${overhead}% overhead${NC}"
        elif [ "$overhead" -le 50 ]; then
            echo -e "${YELLOW}+${overhead}% overhead${NC}"
        else
            echo -e "${RED}+${overhead}% overhead${NC}"
        fi
    fi
}

compare() {
    local title="$1" raw="$2" trs_out="$3" rtk_out="$4" ts_out="$5"
    local raw_t="$6" trs_t="$7" rtk_t="$8" ts_t="$9"
    local raw_b=$(bytes "$raw") trs_b=$(bytes "$trs_out")

    TOTAL=$((TOTAL + 1))

    # Classify as local (<200ms raw) or network (>=200ms raw)
    local is_net=false
    [ -n "$raw_t" ] && [ "$raw_t" -ge 200 ] && is_net=true

    # Accumulate timings into the right bucket
    if [ "$is_net" = true ]; then
        [ -n "$raw_t" ] && RAW_NET_MS=$((RAW_NET_MS + raw_t))
        [ -n "$trs_t" ] && TRS_NET_MS=$((TRS_NET_MS + trs_t))
        NET_TESTS=$((NET_TESTS + 1))
    else
        [ -n "$raw_t" ] && RAW_LOCAL_MS=$((RAW_LOCAL_MS + raw_t))
        [ -n "$trs_t" ] && TRS_LOCAL_MS=$((TRS_LOCAL_MS + trs_t))
        LOCAL_TESTS=$((LOCAL_TESTS + 1))
    fi

    header "$title"

    label "RAW" "$raw_b"
    echo "$raw" | head -6
    [ "$(echo "$raw" | wc -l | tr -d ' ')" -gt 6 ] && echo -e "  ${GRAY}...${NC}"
    [ -n "$raw_t" ] && echo -e "  ${GRAY}⏱  $(time_label "$raw_t")${NC}"

    echo ""
    label "TRS" "$trs_b"
    echo "$trs_out" | head -10
    local trs_pct=$(( raw_b > 0 ? (raw_b - trs_b) * 100 / raw_b : 0 ))
    echo -e "  ${GREEN}↓ ${trs_pct}% reduction ($raw_b → $trs_b bytes)${NC}"
    if [ -n "$trs_t" ]; then
        echo -e "  ${GRAY}⏱  $(time_label "$trs_t")${NC}  $(speed_badge "$trs_t" "$raw_t")"
    fi

    local rtk_b=999999
    if [ "$HAS_RTK" = true ] && [ -n "$rtk_out" ]; then
        rtk_b=$(bytes "$rtk_out")
        if [ -n "$rtk_t" ]; then
            if [ "$is_net" = true ]; then RTK_NET_MS=$((RTK_NET_MS + rtk_t))
            else RTK_LOCAL_MS=$((RTK_LOCAL_MS + rtk_t)); fi
        fi
        echo ""
        label "RTK" "$rtk_b"
        echo "$rtk_out" | head -10
        local rtk_pct=$(( raw_b > 0 ? (raw_b - rtk_b) * 100 / raw_b : 0 ))
        echo -e "  ${YELLOW}↓ ${rtk_pct}% reduction ($raw_b → $rtk_b bytes)${NC}"
        if [ -n "$rtk_t" ]; then
            echo -e "  ${GRAY}⏱  $(time_label "$rtk_t")${NC}  $(speed_badge "$rtk_t" "$raw_t")"
        fi
    fi

    local ts_b=999999
    if [ "$HAS_TS" = true ] && [ -n "$ts_out" ]; then
        ts_b=$(bytes "$ts_out")
        if [ -n "$ts_t" ]; then
            if [ "$is_net" = true ]; then TS_NET_MS=$((TS_NET_MS + ts_t))
            else TS_LOCAL_MS=$((TS_LOCAL_MS + ts_t)); fi
        fi
        echo ""
        label "TS" "$ts_b"
        echo "$ts_out" | head -10
        local ts_pct=$(( raw_b > 0 ? (raw_b - ts_b) * 100 / raw_b : 0 ))
        echo -e "  ${RED}↓ ${ts_pct}% reduction ($raw_b → $ts_b bytes)${NC}"
        if [ -n "$ts_t" ]; then
            echo -e "  ${GRAY}⏱  $(time_label "$ts_t")${NC}  $(speed_badge "$ts_t" "$raw_t")"
        fi
    fi

    # Determine winner among available tools (bytes)
    echo ""
    local best_b=$trs_b best_name="trs"
    if [ "$rtk_b" -lt "$best_b" ]; then best_b=$rtk_b; best_name="rtk"; fi
    if [ "$ts_b" -lt "$best_b" ]; then best_b=$ts_b; best_name="token-saver"; fi

    if [ "$best_b" -eq "$trs_b" ] && { [ "$rtk_b" -eq "$trs_b" ] || [ "$rtk_b" -eq 999999 ]; }; then
        if [ "$ts_b" -eq "$trs_b" ] || [ "$ts_b" -eq 999999 ]; then
            echo -e "  ${GREEN}${BOLD}Winner: trs ($trs_b bytes)${NC}"
            TRS_WINS=$((TRS_WINS + 1))
        fi
    fi

    case "$best_name" in
        trs)
            echo -e "  ${GREEN}${BOLD}Winner: trs ($trs_b bytes)${NC}"
            TRS_WINS=$((TRS_WINS + 1))
            ;;
        rtk)
            echo -e "  ${YELLOW}${BOLD}Winner: rtk ($rtk_b bytes)${NC}"
            RTK_WINS=$((RTK_WINS + 1))
            ;;
        token-saver)
            echo -e "  ${RED}${BOLD}Winner: token-saver ($ts_b bytes)${NC}"
            TS_WINS=$((TS_WINS + 1))
            ;;
    esac

    # Fastest tool (and track speed wins)
    local fastest_t=999999 fastest_name=""
    if [ -n "$trs_t" ] && [ "$trs_t" -lt "$fastest_t" ]; then fastest_t=$trs_t; fastest_name="trs"; fi
    if [ "$HAS_RTK" = true ] && [ -n "$rtk_t" ] && [ "$rtk_t" -lt "$fastest_t" ]; then fastest_t=$rtk_t; fastest_name="rtk"; fi
    if [ "$HAS_TS" = true ] && [ -n "$ts_t" ] && [ "$ts_t" -lt "$fastest_t" ]; then fastest_t=$ts_t; fastest_name="token-saver"; fi
    if [ -n "$fastest_name" ]; then
        echo -e "  ${BOLD}Fastest: ${fastest_name} ($(time_label "$fastest_t"))${NC}"
        case "$fastest_name" in
            trs) TRS_SPEED_WINS=$((TRS_SPEED_WINS + 1)) ;;
            rtk) RTK_SPEED_WINS=$((RTK_SPEED_WINS + 1)) ;;
            token-saver) TS_SPEED_WINS=$((TS_SPEED_WINS + 1)) ;;
        esac
    fi

    pause
}

# For trs-only commands (no rtk equivalent)
compare_trs_only() {
    local title="$1" raw="$2" trs_out="$3" trs_t="$4"
    local raw_b=$(bytes "$raw") trs_b=$(bytes "$trs_out")

    TOTAL=$((TOTAL + 1))
    TRS_WINS=$((TRS_WINS + 1))
    if [ -n "$trs_t" ]; then
        TRS_ONLY_MS=$((TRS_ONLY_MS + trs_t))
        TRS_ONLY_COUNT=$((TRS_ONLY_COUNT + 1))
    fi

    header "$title"

    label "RAW" "$raw_b"
    echo "$raw" | head -6
    [ "$(echo "$raw" | wc -l | tr -d ' ')" -gt 6 ] && echo -e "  ${GRAY}...${NC}"

    echo ""
    label "TRS" "$trs_b"
    echo "$trs_out" | head -10
    local trs_pct=$(( raw_b > 0 ? (raw_b - trs_b) * 100 / raw_b : 0 ))
    echo -e "  ${GREEN}↓ ${trs_pct}% reduction ($raw_b → $trs_b bytes)${NC}"
    [ -n "$trs_t" ] && echo -e "  ${GRAY}⏱  $(time_label "$trs_t")${NC}"
    echo ""
    echo -e "  ${GREEN}${BOLD}trs-only (no rtk equivalent)${NC}"

    pause
}

print_summary() {
    header "SCOREBOARD"
    echo ""
    echo -e "  ${BOLD}Tests:${NC} $TOTAL"
    echo -e "  ${GREEN}${BOLD}trs wins:${NC}          $TRS_WINS"
    [ "$HAS_RTK" = true ] && echo -e "  ${YELLOW}${BOLD}rtk wins:${NC}          $RTK_WINS"
    [ "$HAS_TS" = true ] && echo -e "  ${RED}${BOLD}token-saver wins:${NC} $TS_WINS"
    echo -e "  ${GRAY}${BOLD}ties:${NC}              $TIES"
    echo ""

    local best_wins=$TRS_WINS best_name="trs"
    if [ "$HAS_RTK" = true ] && [ "$RTK_WINS" -gt "$best_wins" ]; then best_wins=$RTK_WINS; best_name="rtk"; fi
    if [ "$HAS_TS" = true ] && [ "$TS_WINS" -gt "$best_wins" ]; then best_wins=$TS_WINS; best_name="token-saver"; fi

    case "$best_name" in
        trs)          echo -e "  ${GREEN}${BOLD}Overall winner: trs${NC}" ;;
        rtk)          echo -e "  ${YELLOW}${BOLD}Overall winner: rtk${NC}" ;;
        token-saver)  echo -e "  ${RED}${BOLD}Overall winner: token-saver${NC}" ;;
    esac

    # Speed wins
    echo ""
    echo -e "  ${BOLD}⏱  Speed wins (per test):${NC}"
    echo -e "  ${GREEN}trs fastest:${NC}       $TRS_SPEED_WINS"
    [ "$HAS_RTK" = true ] && echo -e "  ${YELLOW}rtk fastest:${NC}       $RTK_SPEED_WINS"
    [ "$HAS_TS" = true ] && echo -e "  ${RED}ts fastest:${NC}        $TS_SPEED_WINS"

    # Local tests timing (deterministic, no network variance)
    if [ "$LOCAL_TESTS" -gt 0 ]; then
        echo ""
        echo -e "  ${BOLD}⏱  Local commands ($LOCAL_TESTS tests, deterministic):${NC}"
        echo -e "  ${GRAY}raw (baseline):${NC}    $(time_label "$RAW_LOCAL_MS")"
        echo -e "  ${GREEN}trs:${NC}               $(time_label "$TRS_LOCAL_MS")"
        if [ "$HAS_RTK" = true ] && [ "$RTK_LOCAL_MS" -gt 0 ]; then
            echo -e "  ${YELLOW}rtk:${NC}               $(time_label "$RTK_LOCAL_MS")"
        fi
        if [ "$HAS_TS" = true ] && [ "$TS_LOCAL_MS" -gt 0 ]; then
            echo -e "  ${RED}token-saver:${NC}       $(time_label "$TS_LOCAL_MS")"
        fi
        if [ "$TRS_ONLY_COUNT" -gt 0 ]; then
            echo -e "  ${GRAY}(+ trs-only ${TRS_ONLY_COUNT} tests: $(time_label "$TRS_ONLY_MS"))${NC}"
        fi

        if [ "$RAW_LOCAL_MS" -gt 0 ]; then
            echo ""
            local trs_oh=$(( (TRS_LOCAL_MS - RAW_LOCAL_MS) * 100 / RAW_LOCAL_MS ))
            echo -e "  ${BOLD}Overhead vs raw (local):${NC}"
            echo -e "  ${GREEN}trs:${NC}               ${trs_oh}%"
            if [ "$HAS_RTK" = true ] && [ "$RTK_LOCAL_MS" -gt 0 ]; then
                local rtk_oh=$(( (RTK_LOCAL_MS - RAW_LOCAL_MS) * 100 / RAW_LOCAL_MS ))
                echo -e "  ${YELLOW}rtk:${NC}               ${rtk_oh}%"
            fi
            if [ "$HAS_TS" = true ] && [ "$TS_LOCAL_MS" -gt 0 ]; then
                local ts_oh=$(( (TS_LOCAL_MS - RAW_LOCAL_MS) * 100 / RAW_LOCAL_MS ))
                echo -e "  ${RED}token-saver:${NC}       ${ts_oh}%"
            fi
        fi

        # Fastest local
        echo ""
        local fl=$TRS_LOCAL_MS fn="trs"
        if [ "$HAS_RTK" = true ] && [ "$RTK_LOCAL_MS" -gt 0 ] && [ "$RTK_LOCAL_MS" -lt "$fl" ]; then fl=$RTK_LOCAL_MS; fn="rtk"; fi
        if [ "$HAS_TS" = true ] && [ "$TS_LOCAL_MS" -gt 0 ] && [ "$TS_LOCAL_MS" -lt "$fl" ]; then fl=$TS_LOCAL_MS; fn="token-saver"; fi
        echo -e "  ${BOLD}Fastest (local): ${fn} ($(time_label "$fl"))${NC}"
    fi

    # Network tests timing (shown separately due to high variance)
    if [ "$NET_TESTS" -gt 0 ]; then
        echo ""
        echo -e "  ${GRAY}⏱  Network commands ($NET_TESTS tests, high variance):${NC}"
        echo -e "  ${GRAY}raw:${NC} $(time_label "$RAW_NET_MS")  ${GREEN}trs:${NC} $(time_label "$TRS_NET_MS")"
        if [ "$HAS_RTK" = true ] && [ "$RTK_NET_MS" -gt 0 ]; then
            echo -ne "  ${YELLOW}rtk:${NC} $(time_label "$RTK_NET_MS")"
        fi
        if [ "$HAS_TS" = true ] && [ "$TS_NET_MS" -gt 0 ]; then
            echo -ne "  ${RED}ts:${NC} $(time_label "$TS_NET_MS")"
        fi
        echo ""
    fi
    divider
}

# =============================================================================
echo -e "${BOLD}${GREEN}"
echo "  ╔══════════════════════════════════════════════════════════════╗"
echo "  ║            trs benchmark — Output Comparison                ║"
echo "  ╚══════════════════════════════════════════════════════════════╝"
echo -e "${NC}"
echo -e "  trs: $(trs --version 2>/dev/null || echo 'not found')"
[ "$HAS_RTK" = true ] && echo -e "  rtk: $(rtk --version 2>/dev/null)"
[ "$HAS_TS" = true ] && echo -e "  token-saver: $($TS_WRAP_CMD --version 2>&1 || echo 'v?.?.?')"
echo ""

# Disable git pager for all commands
export GIT_PAGER=cat

# =============================================================================
# SECTION 1: Git commands
# =============================================================================

# 1. git status
timed_run RAW RAW_T git status
timed_run TRS TRS_T timeout 5 trs git status
RTK=""; RTK_T=""; [ "$HAS_RTK" = true ] && timed_run RTK RTK_T timeout 5 rtk git status
TS=""; TS_T=""; [ "$HAS_TS" = true ] && timed_run TS TS_T timeout 10 $TS_WRAP_CMD "git status"
compare "1. git status" "$RAW" "$TRS" "$RTK" "$TS" "$RAW_T" "$TRS_T" "$RTK_T" "$TS_T"

# 2. git diff
timed_run RAW RAW_T git diff HEAD~1 --stat
timed_run TRS TRS_T timeout 5 trs git diff HEAD~1 --stat
RTK=""; RTK_T=""; [ "$HAS_RTK" = true ] && timed_run RTK RTK_T timeout 5 rtk diff HEAD~1 --stat
TS=""; TS_T=""; [ "$HAS_TS" = true ] && timed_run TS TS_T timeout 10 $TS_WRAP_CMD "git diff HEAD~1 --stat"
compare "2. git diff HEAD~1 --stat" "$RAW" "$TRS" "$RTK" "$TS" "$RAW_T" "$TRS_T" "$RTK_T" "$TS_T"

# 3. git log
timed_run RAW RAW_T git log -10
timed_run TRS TRS_T timeout 5 trs git log -10
RTK=""; RTK_T=""; [ "$HAS_RTK" = true ] && timed_run RTK RTK_T timeout 5 rtk git log -10
TS=""; TS_T=""; [ "$HAS_TS" = true ] && timed_run TS TS_T timeout 10 $TS_WRAP_CMD "git log -10"
compare "3. git log -10" "$RAW" "$TRS" "$RTK" "$TS" "$RAW_T" "$TRS_T" "$RTK_T" "$TS_T"

# 4. git branch
timed_run RAW RAW_T git branch -a
timed_run TRS TRS_T timeout 5 trs git branch -a
RTK=""; RTK_T=""; [ "$HAS_RTK" = true ] && timed_run RTK RTK_T timeout 5 rtk git branch -a
TS=""; TS_T=""; [ "$HAS_TS" = true ] && timed_run TS TS_T timeout 10 $TS_WRAP_CMD "git branch -a"
compare "4. git branch -a" "$RAW" "$TRS" "$RTK" "$TS" "$RAW_T" "$TRS_T" "$RTK_T" "$TS_T"

# =============================================================================
# SECTION 2: File system commands
# =============================================================================

# 5. ls -la
timed_run RAW RAW_T ls -la
timed_run TRS TRS_T timeout 5 trs ls -la
RTK=""; RTK_T=""; [ "$HAS_RTK" = true ] && timed_run RTK RTK_T timeout 5 rtk ls -la
TS=""; TS_T=""; [ "$HAS_TS" = true ] && timed_run TS TS_T timeout 10 $TS_WRAP_CMD "ls -la"
compare "5. ls -la" "$RAW" "$TRS" "$RTK" "$TS" "$RAW_T" "$TRS_T" "$RTK_T" "$TS_T"

# 6. find
timed_run RAW RAW_T find src -name "*.rs"
timed_run TRS TRS_T timeout 5 trs find src -name "*.rs"
RTK=""; RTK_T=""; [ "$HAS_RTK" = true ] && timed_run RTK RTK_T timeout 5 rtk find src -name "*.rs"
TS=""; TS_T=""; [ "$HAS_TS" = true ] && timed_run TS TS_T timeout 10 $TS_WRAP_CMD "find src -name '*.rs'"
compare "6. find src -name '*.rs'" "$RAW" "$TRS" "$RTK" "$TS" "$RAW_T" "$TRS_T" "$RTK_T" "$TS_T"

# 7. tree (if available)
if command -v tree &>/dev/null; then
    timed_run RAW RAW_T tree -L 2
    timed_run TRS TRS_T timeout 5 trs tree -L 2
    RTK=""; RTK_T=""; [ "$HAS_RTK" = true ] && timed_run RTK RTK_T timeout 5 rtk tree -L 2
    TS=""; TS_T=""; [ "$HAS_TS" = true ] && timed_run TS TS_T timeout 10 $TS_WRAP_CMD "tree -L 2"
    compare "7. tree -L 2" "$RAW" "$TRS" "$RTK" "$TS" "$RAW_T" "$TRS_T" "$RTK_T" "$TS_T"
fi

# =============================================================================
# SECTION 3: Environment & system
# =============================================================================

# 8. env
timed_run RAW RAW_T env
timed_run TRS TRS_T timeout 5 trs env
RTK=""; RTK_T=""; [ "$HAS_RTK" = true ] && timed_run RTK RTK_T timeout 5 rtk env
TS=""; TS_T=""; [ "$HAS_TS" = true ] && timed_run TS TS_T timeout 10 $TS_WRAP_CMD "env"
compare "8. env" "$RAW" "$TRS" "$RTK" "$TS" "$RAW_T" "$TRS_T" "$RTK_T" "$TS_T"

# 9. wc
timed_run RAW RAW_T wc src/main.rs src/cli.rs src/commands.rs
timed_run TRS TRS_T timeout 5 trs wc src/main.rs src/cli.rs src/commands.rs
RTK=""; RTK_T=""; [ "$HAS_RTK" = true ] && timed_run RTK RTK_T timeout 5 rtk wc src/main.rs src/cli.rs src/commands.rs
TS=""; TS_T=""; [ "$HAS_TS" = true ] && timed_run TS TS_T timeout 10 $TS_WRAP_CMD "wc src/main.rs src/cli.rs src/commands.rs"
compare "9. wc (3 files)" "$RAW" "$TRS" "$RTK" "$TS" "$RAW_T" "$TRS_T" "$RTK_T" "$TS_T"

# =============================================================================
# SECTION 4: Search (grep)
# =============================================================================

# 10. grep
timed_run RAW RAW_T grep -rn "pub fn " src/main.rs src/cli.rs src/commands.rs
timed_run TRS TRS_T timeout 5 trs grep -rn "pub fn " src/main.rs src/cli.rs src/commands.rs
RTK=""; RTK_T=""; [ "$HAS_RTK" = true ] && timed_run RTK RTK_T timeout 5 rtk grep -rn "pub fn " src/main.rs src/cli.rs src/commands.rs
TS=""; TS_T=""; [ "$HAS_TS" = true ] && timed_run TS TS_T timeout 10 $TS_WRAP_CMD "grep -rn 'pub fn ' src/main.rs src/cli.rs src/commands.rs"
compare "10. grep -rn 'pub fn' (3 files)" "$RAW" "$TRS" "$RTK" "$TS" "$RAW_T" "$TRS_T" "$RTK_T" "$TS_T"

# =============================================================================
# SECTION 5: Build & test (cargo)
# =============================================================================

# 11. cargo test (small subset)
timed_run RAW RAW_T env RTK_DISABLED=1 cargo test test_defaults --
timed_run TRS TRS_T timeout 30 trs cargo test test_defaults --
RTK=""; RTK_T=""; [ "$HAS_RTK" = true ] && timed_run RTK RTK_T timeout 30 rtk cargo test test_defaults --
TS=""; TS_T=""; [ "$HAS_TS" = true ] && timed_run TS TS_T timeout 30 $TS_WRAP_CMD "cargo test test_defaults --"
compare "11. cargo test (1 test)" "$RAW" "$TRS" "$RTK" "$TS" "$RAW_T" "$TRS_T" "$RTK_T" "$TS_T"

# 12. cargo build (should be cached / fast)
timed_run RAW RAW_T env RTK_DISABLED=1 cargo build
timed_run TRS TRS_T timeout 30 trs cargo build
RTK=""; RTK_T=""; [ "$HAS_RTK" = true ] && timed_run RTK RTK_T timeout 30 rtk cargo build
TS=""; TS_T=""; [ "$HAS_TS" = true ] && timed_run TS TS_T timeout 30 $TS_WRAP_CMD "cargo build"
compare "12. cargo build" "$RAW" "$TRS" "$RTK" "$TS" "$RAW_T" "$TRS_T" "$RTK_T" "$TS_T"

# =============================================================================
# SECTION 6: GitHub CLI (if available)
# =============================================================================

if command -v gh &>/dev/null; then
    # 13. gh pr list
    timed_run RAW RAW_T gh pr list -R cli/cli --limit 5
    if [ "$RAW" != "(timeout)" ] && [ -n "$RAW" ]; then
        timed_run TRS TRS_T timeout 10 trs gh pr list -R cli/cli --limit 5
        RTK=""; RTK_T=""; [ "$HAS_RTK" = true ] && timed_run RTK RTK_T timeout 10 rtk gh pr list -R cli/cli --limit 5
        TS=""; TS_T=""; [ "$HAS_TS" = true ] && timed_run TS TS_T timeout 15 $TS_WRAP_CMD "gh pr list -R cli/cli --limit 5"
        compare "13. gh pr list (cli/cli)" "$RAW" "$TRS" "$RTK" "$TS" "$RAW_T" "$TRS_T" "$RTK_T" "$TS_T"
    fi

    # 14. gh run list
    timed_run RAW RAW_T gh run list -R cli/cli --limit 5
    if [ "$RAW" != "(timeout)" ] && [ -n "$RAW" ]; then
        timed_run TRS TRS_T timeout 10 trs gh run list -R cli/cli --limit 5
        RTK=""; RTK_T=""; [ "$HAS_RTK" = true ] && timed_run RTK RTK_T timeout 10 rtk gh run list -R cli/cli --limit 5
        TS=""; TS_T=""; [ "$HAS_TS" = true ] && timed_run TS TS_T timeout 15 $TS_WRAP_CMD "gh run list -R cli/cli --limit 5"
        compare "14. gh run list (cli/cli)" "$RAW" "$TRS" "$RTK" "$TS" "$RAW_T" "$TRS_T" "$RTK_T" "$TS_T"
    fi
fi

# =============================================================================
# SECTION 7: Network (curl)
# =============================================================================

# 15. curl -I (HTTP headers)
timed_run RAW RAW_T curl -sI https://httpbin.org/get
if [ "$RAW" != "(timeout)" ] && [ -n "$RAW" ]; then
    timed_run TRS TRS_T timeout 10 trs curl -I https://httpbin.org/get
    RTK=""; RTK_T=""; [ "$HAS_RTK" = true ] && timed_run RTK RTK_T timeout 10 rtk curl -I https://httpbin.org/get
    TS=""; TS_T=""; [ "$HAS_TS" = true ] && timed_run TS TS_T timeout 15 $TS_WRAP_CMD "curl -sI https://httpbin.org/get"
    compare "15. curl -I httpbin.org" "$RAW" "$TRS" "$RTK" "$TS" "$RAW_T" "$TRS_T" "$RTK_T" "$TS_T"
fi

# =============================================================================
# SECTION 8: trs-only features (no rtk equivalent)
# =============================================================================

# 16. trs json (structure without values)
JSON_SAMPLE='{"users":[{"id":1,"name":"Alice","email":"a@b.com"},{"id":2,"name":"Bob","email":"c@d.com"},{"id":3,"name":"Charlie","email":"e@f.com"},{"id":4,"name":"Dave","email":"g@h.com"},{"id":5,"name":"Eve","email":"i@j.com"},{"id":6,"name":"Frank","email":"k@l.com"}],"meta":{"page":1,"total":100,"cursor":"abc123"}}'
RAW="$JSON_SAMPLE"
_s=$(now_ms); TRS=$(echo "$JSON_SAMPLE" | timeout 5 trs json 2>/dev/null || echo "(timeout)"); TRS_T=$(( $(now_ms) - _s ))
compare_trs_only "16. trs json (structure)" "$RAW" "$TRS" "$TRS_T"

# 17. trs read --aggressive (signatures only)
RAW=$(cat src/router/handlers/err.rs 2>/dev/null)
timed_run TRS TRS_T timeout 5 trs read src/router/handlers/err.rs -l aggressive
compare_trs_only "17. trs read -l aggressive (err.rs)" "$RAW" "$TRS" "$TRS_T"

# 18. trs read --minimal (strip comments)
RAW=$(cat src/router/handlers/read.rs 2>/dev/null)
timed_run TRS TRS_T timeout 5 trs read src/router/handlers/read.rs -l minimal
compare_trs_only "18. trs read -l minimal (read.rs)" "$RAW" "$TRS" "$TRS_T"

# 19. trs err (error filter)
ERR_INPUT="line 1 ok
line 2 ok
error: something broke
  at module.rs:42
line 5 ok
warning: deprecated API
line 7 ok
line 8 ok
ERROR: fatal crash
line 10 ok"
RAW="$ERR_INPUT"
_s=$(now_ms); TRS=$(echo "$ERR_INPUT" | timeout 5 trs clean --no-ansi 2>/dev/null || echo "(timeout)"); TRS_T=$(( $(now_ms) - _s ))
compare_trs_only "19. trs clean (text cleanup)" "$RAW" "$TRS" "$TRS_T"

# =============================================================================
# SECTION 9: Docker (if available)
# =============================================================================

if command -v docker &>/dev/null; then
    timed_run RAW RAW_T docker ps
    if [ "$RAW" != "(timeout)" ] && [ "$(echo "$RAW" | wc -l | tr -d ' ')" -gt 1 ]; then
        timed_run TRS TRS_T timeout 5 trs docker ps
        RTK=""; RTK_T=""; [ "$HAS_RTK" = true ] && timed_run RTK RTK_T timeout 5 rtk docker ps
        TS=""; TS_T=""; [ "$HAS_TS" = true ] && timed_run TS TS_T timeout 10 $TS_WRAP_CMD "docker ps"
        compare "20. docker ps" "$RAW" "$TRS" "$RTK" "$TS" "$RAW_T" "$TRS_T" "$RTK_T" "$TS_T"
    fi
fi

# =============================================================================
# Summary
# =============================================================================
print_summary
