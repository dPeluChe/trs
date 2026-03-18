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

# Score tracking
TRS_WINS=0
RTK_WINS=0
TIES=0
TOTAL=0

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

compare() {
    local title="$1" raw="$2" trs_out="$3" rtk_out="$4"
    local raw_b=$(bytes "$raw") trs_b=$(bytes "$trs_out")

    TOTAL=$((TOTAL + 1))

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
            TRS_WINS=$((TRS_WINS + 1))
        elif [ "$rtk_b" -lt "$trs_b" ]; then
            echo -e "  ${YELLOW}${BOLD}Winner: rtk ($rtk_b vs $trs_b bytes)${NC}"
            RTK_WINS=$((RTK_WINS + 1))
        else
            echo -e "  ${GRAY}${BOLD}Tie${NC}"
            TIES=$((TIES + 1))
        fi
    fi

    pause
}

# For trs-only commands (no rtk equivalent)
compare_trs_only() {
    local title="$1" raw="$2" trs_out="$3"
    local raw_b=$(bytes "$raw") trs_b=$(bytes "$trs_out")

    TOTAL=$((TOTAL + 1))
    TRS_WINS=$((TRS_WINS + 1))

    header "$title"

    label "RAW" "$raw_b"
    echo "$raw" | head -6
    [ "$(echo "$raw" | wc -l | tr -d ' ')" -gt 6 ] && echo -e "  ${GRAY}...${NC}"

    echo ""
    label "TRS" "$trs_b"
    echo "$trs_out" | head -10
    local trs_pct=$(( raw_b > 0 ? (raw_b - trs_b) * 100 / raw_b : 0 ))
    echo -e "  ${GREEN}↓ ${trs_pct}% reduction ($raw_b → $trs_b bytes)${NC}"
    echo ""
    echo -e "  ${GREEN}${BOLD}trs-only (no rtk equivalent)${NC}"

    pause
}

print_summary() {
    header "SCOREBOARD"
    echo ""
    echo -e "  ${BOLD}Tests:${NC} $TOTAL"
    echo -e "  ${GREEN}${BOLD}trs wins:${NC}  $TRS_WINS"
    if [ "$HAS_RTK" = true ]; then
        echo -e "  ${YELLOW}${BOLD}rtk wins:${NC}  $RTK_WINS"
    fi
    echo -e "  ${GRAY}${BOLD}ties:${NC}      $TIES"
    echo ""

    if [ "$TRS_WINS" -gt "$RTK_WINS" ]; then
        echo -e "  ${GREEN}${BOLD}Overall winner: trs${NC}"
    elif [ "$RTK_WINS" -gt "$TRS_WINS" ]; then
        echo -e "  ${YELLOW}${BOLD}Overall winner: rtk${NC}"
    else
        echo -e "  ${GRAY}${BOLD}Overall: tie${NC}"
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
echo ""

# Disable git pager for all commands
export GIT_PAGER=cat

# =============================================================================
# SECTION 1: Git commands
# =============================================================================

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

# =============================================================================
# SECTION 2: File system commands
# =============================================================================

# 5. ls -la
RAW=$(ls -la 2>/dev/null)
TRS=$(timeout 5 trs ls -la 2>/dev/null || echo "(timeout)")
RTK=""; [ "$HAS_RTK" = true ] && RTK=$(timeout 5 rtk ls -la 2>/dev/null || echo "(timeout)")
compare "5. ls -la" "$RAW" "$TRS" "$RTK"

# 6. find
RAW=$(find src -name "*.rs" 2>/dev/null)
TRS=$(timeout 5 trs find src -name "*.rs" 2>/dev/null || echo "(timeout)")
RTK=""; [ "$HAS_RTK" = true ] && RTK=$(timeout 5 rtk find src -name "*.rs" 2>/dev/null || echo "(timeout)")
compare "6. find src -name '*.rs'" "$RAW" "$TRS" "$RTK"

# 7. tree (if available)
if command -v tree &>/dev/null; then
    RAW=$(tree -L 2 2>/dev/null)
    TRS=$(timeout 5 trs tree -L 2 2>/dev/null || echo "(timeout)")
    RTK=""; [ "$HAS_RTK" = true ] && RTK=$(timeout 5 rtk tree -L 2 2>/dev/null || echo "(timeout)")
    compare "7. tree -L 2" "$RAW" "$TRS" "$RTK"
fi

# =============================================================================
# SECTION 3: Environment & system
# =============================================================================

# 8. env
RAW=$(env 2>/dev/null)
TRS=$(timeout 5 trs env 2>/dev/null || echo "(timeout)")
RTK=""; [ "$HAS_RTK" = true ] && RTK=$(timeout 5 rtk env 2>/dev/null || echo "(timeout)")
compare "8. env" "$RAW" "$TRS" "$RTK"

# 9. wc
RAW=$(wc src/main.rs src/cli.rs src/commands.rs 2>/dev/null)
TRS=$(timeout 5 trs wc src/main.rs src/cli.rs src/commands.rs 2>/dev/null || echo "(timeout)")
RTK=""; [ "$HAS_RTK" = true ] && RTK=$(timeout 5 rtk wc src/main.rs src/cli.rs src/commands.rs 2>/dev/null || echo "(timeout)")
compare "9. wc (3 files)" "$RAW" "$TRS" "$RTK"

# =============================================================================
# SECTION 4: Search (grep)
# =============================================================================

# 10. grep
RAW=$(grep -rn "pub fn " src/main.rs src/cli.rs src/commands.rs 2>/dev/null || true)
TRS=$(timeout 5 trs grep -rn "pub fn " src/main.rs src/cli.rs src/commands.rs 2>/dev/null || echo "(timeout)")
RTK=""; [ "$HAS_RTK" = true ] && RTK=$(timeout 5 rtk grep -rn "pub fn " src/main.rs src/cli.rs src/commands.rs 2>/dev/null || echo "(timeout)")
compare "10. grep -rn 'pub fn' (3 files)" "$RAW" "$TRS" "$RTK"

# =============================================================================
# SECTION 5: Build & test (cargo)
# =============================================================================

# 11. cargo test (small subset)
RAW=$(RTK_DISABLED=1 command cargo test test_defaults -- 2>&1 || true)
TRS=$(timeout 30 trs cargo test test_defaults -- 2>/dev/null || echo "(timeout)")
RTK=""; [ "$HAS_RTK" = true ] && RTK=$(timeout 30 rtk cargo test test_defaults -- 2>/dev/null || echo "(timeout)")
compare "11. cargo test (1 test)" "$RAW" "$TRS" "$RTK"

# 12. cargo build (should be cached / fast)
RAW=$(RTK_DISABLED=1 command cargo build 2>&1 || true)
TRS=$(timeout 30 trs cargo build 2>/dev/null || echo "(timeout)")
RTK=""; [ "$HAS_RTK" = true ] && RTK=$(timeout 30 rtk cargo build 2>/dev/null || echo "(timeout)")
compare "12. cargo build" "$RAW" "$TRS" "$RTK"

# =============================================================================
# SECTION 6: GitHub CLI (if available)
# =============================================================================

if command -v gh &>/dev/null; then
    # 13. gh pr list
    RAW=$(gh pr list -R cli/cli --limit 5 2>/dev/null || echo "(not available)")
    if [ "$RAW" != "(not available)" ]; then
        TRS=$(timeout 10 trs gh pr list -R cli/cli --limit 5 2>/dev/null || echo "(timeout)")
        RTK=""; [ "$HAS_RTK" = true ] && RTK=$(timeout 10 rtk gh pr list -R cli/cli --limit 5 2>/dev/null || echo "(timeout)")
        compare "13. gh pr list (cli/cli)" "$RAW" "$TRS" "$RTK"
    fi

    # 14. gh run list
    RAW=$(gh run list -R cli/cli --limit 5 2>/dev/null || echo "(not available)")
    if [ "$RAW" != "(not available)" ]; then
        TRS=$(timeout 10 trs gh run list -R cli/cli --limit 5 2>/dev/null || echo "(timeout)")
        RTK=""; [ "$HAS_RTK" = true ] && RTK=$(timeout 10 rtk gh run list -R cli/cli --limit 5 2>/dev/null || echo "(timeout)")
        compare "14. gh run list (cli/cli)" "$RAW" "$TRS" "$RTK"
    fi
fi

# =============================================================================
# SECTION 7: Network (curl)
# =============================================================================

# 15. curl -I (HTTP headers)
RAW=$(curl -sI https://httpbin.org/get 2>/dev/null || echo "(not available)")
if [ "$RAW" != "(not available)" ] && [ -n "$RAW" ]; then
    TRS=$(timeout 10 trs curl -I https://httpbin.org/get 2>/dev/null || echo "(timeout)")
    RTK=""; [ "$HAS_RTK" = true ] && RTK=$(timeout 10 rtk curl -I https://httpbin.org/get 2>/dev/null || echo "(timeout)")
    compare "15. curl -I httpbin.org" "$RAW" "$TRS" "$RTK"
fi

# =============================================================================
# SECTION 8: trs-only features (no rtk equivalent)
# =============================================================================

# 16. trs json (structure without values)
JSON_SAMPLE='{"users":[{"id":1,"name":"Alice","email":"a@b.com"},{"id":2,"name":"Bob","email":"c@d.com"},{"id":3,"name":"Charlie","email":"e@f.com"},{"id":4,"name":"Dave","email":"g@h.com"},{"id":5,"name":"Eve","email":"i@j.com"},{"id":6,"name":"Frank","email":"k@l.com"}],"meta":{"page":1,"total":100,"cursor":"abc123"}}'
RAW="$JSON_SAMPLE"
TRS=$(echo "$JSON_SAMPLE" | timeout 5 trs json 2>/dev/null || echo "(timeout)")
compare_trs_only "16. trs json (structure)" "$RAW" "$TRS"

# 17. trs read --aggressive (signatures only)
RAW=$(cat src/router/handlers/err.rs 2>/dev/null)
TRS=$(timeout 5 trs read src/router/handlers/err.rs -l aggressive 2>/dev/null || echo "(timeout)")
compare_trs_only "17. trs read -l aggressive (err.rs)" "$RAW" "$TRS"

# 18. trs read --minimal (strip comments)
RAW=$(cat src/router/handlers/read.rs 2>/dev/null)
TRS=$(timeout 5 trs read src/router/handlers/read.rs -l minimal 2>/dev/null || echo "(timeout)")
compare_trs_only "18. trs read -l minimal (read.rs)" "$RAW" "$TRS"

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
TRS=$(echo "$ERR_INPUT" | timeout 5 trs clean --no-ansi 2>/dev/null || echo "(timeout)")
compare_trs_only "19. trs clean (text cleanup)" "$RAW" "$TRS"

# =============================================================================
# SECTION 9: Docker (if available)
# =============================================================================

if command -v docker &>/dev/null; then
    RAW=$(docker ps 2>/dev/null || echo "(not available)")
    if [ "$RAW" != "(not available)" ] && [ "$(echo "$RAW" | wc -l | tr -d ' ')" -gt 1 ]; then
        TRS=$(timeout 5 trs docker ps 2>/dev/null || echo "(timeout)")
        RTK=""; [ "$HAS_RTK" = true ] && RTK=$(timeout 5 rtk docker ps 2>/dev/null || echo "(timeout)")
        compare "20. docker ps" "$RAW" "$TRS" "$RTK"
    fi
fi

# =============================================================================
# Summary
# =============================================================================
print_summary
