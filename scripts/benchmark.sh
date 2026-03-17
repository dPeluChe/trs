#!/bin/bash
# =============================================================================
# trs benchmark — Compare raw, trs, and rtk output
# Usage: ./scripts/benchmark.sh [--all]
#   Without --all: pauses between each test (press Enter to continue)
#   With --all: runs all tests without pausing
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

RUN_ALL=false
if [ "$1" = "--all" ]; then RUN_ALL=true; fi

HAS_RTK=false
if command -v rtk &>/dev/null; then HAS_RTK=true; fi

divider() { echo -e "\n${GRAY}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"; }
header()  { divider; echo -e "${BOLD}${BLUE}  $1${NC}"; divider; }
label()   { echo -e "  ${CYAN}[$1]${NC} ${GRAY}($2 bytes)${NC}"; }

bytes() { echo -n "$1" | wc -c | tr -d ' '; }

pause() {
    if [ "$RUN_ALL" = false ]; then
        echo -e "\n  ${GRAY}Press Enter for next test (or q to quit)...${NC}"
        read -r input
        if [ "$input" = "q" ]; then exit 0; fi
    fi
}

compare() {
    local title="$1" raw="$2" trs_out="$3" rtk_out="$4"
    local raw_b=$(bytes "$raw") trs_b=$(bytes "$trs_out")

    header "$title"

    label "RAW" "$raw_b"
    echo "$raw" | head -6
    [ "$(echo "$raw" | wc -l | tr -d ' ')" -gt 6 ] && echo -e "  ${GRAY}...${NC}"

    echo ""
    label "TRS" "$trs_b"
    echo "$trs_out" | head -10
    local trs_pct=$(( raw_b > 0 ? (raw_b - trs_b) * 100 / raw_b : 0 ))
    echo -e "  ${GREEN}↓ ${trs_pct}% reduction ($raw_b → $trs_b bytes)${NC}"

    if [ "$HAS_RTK" = true ] && [ -n "$rtk_out" ]; then
        local rtk_b=$(bytes "$rtk_out")
        echo ""
        label "RTK" "$rtk_b"
        echo "$rtk_out" | head -10
        local rtk_pct=$(( raw_b > 0 ? (raw_b - rtk_b) * 100 / raw_b : 0 ))
        echo -e "  ${YELLOW}↓ ${rtk_pct}% reduction ($raw_b → $rtk_b bytes)${NC}"

        echo ""
        if [ "$trs_b" -lt "$rtk_b" ]; then
            echo -e "  ${GREEN}${BOLD}Winner: trs ($trs_b vs $rtk_b bytes)${NC}"
        elif [ "$rtk_b" -lt "$trs_b" ]; then
            echo -e "  ${YELLOW}${BOLD}Winner: rtk ($rtk_b vs $trs_b bytes)${NC}"
        else
            echo -e "  ${GRAY}${BOLD}Tie${NC}"
        fi
    fi

    pause
}

# =============================================================================
echo -e "${BOLD}${GREEN}"
echo "  ╔══════════════════════════════════════════════════════════════╗"
echo "  ║            trs benchmark — Output Comparison                ║"
echo "  ╚══════════════════════════════════════════════════════════════╝"
echo -e "${NC}"
echo -e "  trs: $(trs --version 2>/dev/null || echo 'not found')"
[ "$HAS_RTK" = true ] && echo -e "  rtk: $(rtk --version 2>/dev/null)"
echo ""

# Disable git pager for all commands
export GIT_PAGER=cat

# 1. git status
RAW=$(git status 2>/dev/null)
TRS=$(timeout 5 trs git status 2>/dev/null || echo "(timeout)")
RTK=""; [ "$HAS_RTK" = true ] && RTK=$(timeout 5 rtk git status 2>/dev/null || echo "(timeout)")
compare "1. git status" "$RAW" "$TRS" "$RTK"

# 2. git diff
RAW=$(git diff HEAD~1 --stat 2>/dev/null)
TRS=$(timeout 5 trs git diff HEAD~1 --stat 2>/dev/null || echo "(timeout)")
RTK=""; [ "$HAS_RTK" = true ] && RTK=$(timeout 5 rtk diff HEAD~1 --stat 2>/dev/null || echo "(timeout)")
compare "2. git diff HEAD~1 --stat" "$RAW" "$TRS" "$RTK"

# 3. git log
RAW=$(git log -10 2>/dev/null)
TRS=$(timeout 5 trs git log -10 2>/dev/null || echo "(timeout)")
RTK=""; [ "$HAS_RTK" = true ] && RTK=$(timeout 5 rtk git log -10 2>/dev/null || echo "(timeout)")
compare "3. git log -10" "$RAW" "$TRS" "$RTK"

# 4. git branch
RAW=$(git branch -a 2>/dev/null)
TRS=$(timeout 5 trs git branch -a 2>/dev/null || echo "(timeout)")
RTK=""; [ "$HAS_RTK" = true ] && RTK=$(timeout 5 rtk git branch -a 2>/dev/null || echo "(timeout)")
compare "4. git branch -a" "$RAW" "$TRS" "$RTK"

# 5. ls -la
RAW=$(ls -la 2>/dev/null)
TRS=$(timeout 5 trs ls -la 2>/dev/null || echo "(timeout)")
RTK=""; [ "$HAS_RTK" = true ] && RTK=$(timeout 5 rtk ls -la 2>/dev/null || echo "(timeout)")
compare "5. ls -la" "$RAW" "$TRS" "$RTK"

# 6. env
RAW=$(env 2>/dev/null)
TRS=$(timeout 5 trs env 2>/dev/null || echo "(timeout)")
RTK=""; [ "$HAS_RTK" = true ] && RTK=$(timeout 5 rtk env 2>/dev/null || echo "(timeout)")
compare "6. env" "$RAW" "$TRS" "$RTK"

# 7. find
RAW=$(find src -name "*.rs" 2>/dev/null)
TRS=$(timeout 5 trs find src -name "*.rs" 2>/dev/null || echo "(timeout)")
RTK=""; [ "$HAS_RTK" = true ] && RTK=$(timeout 5 rtk find src -name "*.rs" 2>/dev/null || echo "(timeout)")
compare "7. find src -name '*.rs'" "$RAW" "$TRS" "$RTK"

# 8. tree (if available)
if command -v tree &>/dev/null; then
    RAW=$(tree -L 2 2>/dev/null)
    TRS=$(timeout 5 trs tree -L 2 2>/dev/null || echo "(timeout)")
    RTK=""; [ "$HAS_RTK" = true ] && RTK=$(timeout 5 rtk tree -L 2 2>/dev/null || echo "(timeout)")
    compare "8. tree -L 2" "$RAW" "$TRS" "$RTK"
fi

# Summary
header "DONE"
echo -e "  ${BOLD}Benchmark complete.${NC}"
echo -e "  ${GRAY}Run with --all to skip pauses: ./scripts/benchmark.sh --all${NC}"
divider
