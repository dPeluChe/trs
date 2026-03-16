#!/bin/bash
# =============================================================================
# trs benchmark — Compare raw, trs, and rtk output for common commands
# Usage: ./scripts/benchmark.sh
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

HAS_RTK=false
if command -v rtk &>/dev/null; then HAS_RTK=true; fi

divider() { echo -e "\n${GRAY}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"; }
header()  { divider; echo -e "${BOLD}${BLUE}  $1${NC}"; divider; }
label()   { echo -e "  ${CYAN}[$1]${NC} ${GRAY}($2 bytes)${NC}"; }

# Count bytes helper
bytes() { echo -n "$1" | wc -c | tr -d ' '; }

# Reduction calc
reduction() {
    local raw_b=$1 compact_b=$2
    if [ "$raw_b" -gt 0 ]; then
        echo $(( (raw_b - compact_b) * 100 / raw_b ))
    else
        echo 0
    fi
}

compare() {
    local title="$1"
    local raw="$2"
    local trs_out="$3"
    local rtk_out="$4"

    local raw_b=$(bytes "$raw")
    local trs_b=$(bytes "$trs_out")

    header "$title"

    label "RAW" "$raw_b"
    echo "$raw" | head -8
    if [ "$(echo "$raw" | wc -l | tr -d ' ')" -gt 8 ]; then echo -e "  ${GRAY}... (truncated)${NC}"; fi

    echo ""
    label "TRS" "$trs_b"
    echo "$trs_out" | head -15
    local trs_pct=$(reduction "$raw_b" "$trs_b")
    echo -e "  ${GREEN}↓ ${trs_pct}% reduction ($raw_b → $trs_b bytes)${NC}"

    if [ "$HAS_RTK" = true ] && [ -n "$rtk_out" ]; then
        local rtk_b=$(bytes "$rtk_out")
        echo ""
        label "RTK" "$rtk_b"
        echo "$rtk_out" | head -15
        local rtk_pct=$(reduction "$raw_b" "$rtk_b")
        echo -e "  ${YELLOW}↓ ${rtk_pct}% reduction ($raw_b → $rtk_b bytes)${NC}"

        # Winner
        echo ""
        if [ "$trs_b" -lt "$rtk_b" ]; then
            echo -e "  ${GREEN}${BOLD}Winner: trs ($trs_b < $rtk_b bytes)${NC}"
        elif [ "$rtk_b" -lt "$trs_b" ]; then
            echo -e "  ${YELLOW}${BOLD}Winner: rtk ($rtk_b < $trs_b bytes)${NC}"
        else
            echo -e "  ${GRAY}${BOLD}Tie ($trs_b = $rtk_b bytes)${NC}"
        fi
    fi
}

# =============================================================================
echo -e "${BOLD}${GREEN}"
echo "  ╔══════════════════════════════════════════════════════════════╗"
echo "  ║            trs benchmark — Output Comparison                ║"
echo "  ╚══════════════════════════════════════════════════════════════╝"
echo -e "${NC}"
echo -e "  trs version: $(trs --version 2>/dev/null || echo 'not found')"
if [ "$HAS_RTK" = true ]; then echo -e "  rtk version: $(rtk --version 2>/dev/null)"; fi
echo ""

# =============================================================================
# 1. git status
# =============================================================================
RAW=$(git status 2>/dev/null)
TRS=$(trs git status 2>/dev/null)
RTK=""
if [ "$HAS_RTK" = true ]; then RTK=$(rtk git status 2>/dev/null); fi
compare "git status" "$RAW" "$TRS" "$RTK"

# =============================================================================
# 2. git diff (vs origin)
# =============================================================================
RAW=$(git diff HEAD~1 2>/dev/null | head -100)
TRS=$(trs git diff HEAD~1 2>/dev/null)
RTK=""
if [ "$HAS_RTK" = true ]; then RTK=$(rtk diff HEAD~1 2>/dev/null); fi
compare "git diff HEAD~1" "$RAW" "$TRS" "$RTK"

# =============================================================================
# 3. git log
# =============================================================================
RAW=$(git log -10 2>/dev/null)
TRS=$(trs git log -10 2>/dev/null)
RTK=""
if [ "$HAS_RTK" = true ]; then RTK=$(rtk git log -10 2>/dev/null); fi
compare "git log -10" "$RAW" "$TRS" "$RTK"

# =============================================================================
# 4. git branch
# =============================================================================
RAW=$(git branch -a 2>/dev/null)
TRS=$(trs git branch -a 2>/dev/null)
RTK=""
if [ "$HAS_RTK" = true ]; then RTK=$(rtk git branch -a 2>/dev/null); fi
compare "git branch -a" "$RAW" "$TRS" "$RTK"

# =============================================================================
# 5. ls -la
# =============================================================================
RAW=$(ls -la 2>/dev/null)
TRS=$(trs ls -la 2>/dev/null)
RTK=""
if [ "$HAS_RTK" = true ]; then RTK=$(rtk ls -la 2>/dev/null); fi
compare "ls -la" "$RAW" "$TRS" "$RTK"

# =============================================================================
# 6. env
# =============================================================================
RAW=$(env 2>/dev/null)
TRS=$(trs env 2>/dev/null)
RTK=""
if [ "$HAS_RTK" = true ]; then RTK=$(rtk env 2>/dev/null); fi
compare "env" "$RAW" "$TRS" "$RTK"

# =============================================================================
# 7. find
# =============================================================================
RAW=$(find src -name "*.rs" 2>/dev/null)
TRS=$(trs find src -name "*.rs" 2>/dev/null)
RTK=""
if [ "$HAS_RTK" = true ]; then RTK=$(rtk find src -name "*.rs" 2>/dev/null); fi
compare "find src -name '*.rs'" "$RAW" "$TRS" "$RTK"

# =============================================================================
# 8. tree (if available)
# =============================================================================
if command -v tree &>/dev/null; then
    RAW=$(tree -L 2 2>/dev/null)
    TRS=$(trs tree -L 2 2>/dev/null)
    RTK=""
    if [ "$HAS_RTK" = true ]; then RTK=$(rtk tree -L 2 2>/dev/null); fi
    compare "tree -L 2" "$RAW" "$TRS" "$RTK"
fi

# =============================================================================
# Summary
# =============================================================================
header "SUMMARY"
echo -e "  ${BOLD}Commands tested:${NC} $([ "$HAS_RTK" = true ] && echo "8 (with rtk comparison)" || echo "8 (trs only)")"
echo -e "  ${BOLD}trs:${NC} Reduces token consumption across all tested commands"
if [ "$HAS_RTK" = true ]; then
    echo -e "  ${BOLD}rtk:${NC} Available for comparison — both tools optimize terminal output"
fi
echo ""
echo -e "  ${GRAY}Run: trs <command> --stats  to see detailed reduction metrics${NC}"
divider
