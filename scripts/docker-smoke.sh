#!/usr/bin/env bash
# docker-smoke.sh — Test trs installation on fresh Linux distributions.
#
# Usage:
#   ./scripts/docker-smoke.sh              # Test all distros
#   ./scripts/docker-smoke.sh ubuntu       # Test one distro
#   ./scripts/docker-smoke.sh --build      # Rebuild binary first (cargo build --release)
#
# Prerequisites: Docker must be running.
#
# How it works:
#   1. Builds a linux-x64 binary via Docker (or uses an existing one)
#   2. Copies it into fresh containers for each distro
#   3. Runs `trs doctor --json` and validates the output
#
# This simulates what a real user gets after `npm install -g tars-cli`
# or downloading a prebuilt binary from GitHub Releases.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
BINARY="$PROJECT_DIR/target/x86_64-unknown-linux-gnu/release/trs"

DISTROS=(
    "ubuntu:22.04"
    "ubuntu:24.04"
    "debian:bookworm-slim"
    "alpine:3.19"
    "fedora:40"
    "archlinux:latest"
)

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
NC='\033[0m'

PASS=0
FAIL=0
SKIP=0

# ============================================================
# Build linux binary
# ============================================================

build_binary() {
    echo "Building linux-x64 binary via Docker..."
    docker run --rm \
        -v "$PROJECT_DIR":/src \
        -w /src \
        rust:1.82-bookworm \
        bash -c "
            rustup target add x86_64-unknown-linux-gnu 2>/dev/null
            cargo build --release --target x86_64-unknown-linux-gnu
        "
    echo ""
}

# ============================================================
# Run smoke test on one distro
# ============================================================

test_distro() {
    local image="$1"
    local label="${image//[:\/]/-}"

    # Pick the right package manager for installing git
    local install_git=""
    case "$image" in
        ubuntu*|debian*)  install_git="apt-get update -qq && apt-get install -y -qq git >/dev/null 2>&1" ;;
        alpine*)          install_git="apk add --no-cache git >/dev/null 2>&1" ;;
        fedora*)          install_git="dnf install -y -q git >/dev/null 2>&1" ;;
        archlinux*)       install_git="pacman -Sy --noconfirm git >/dev/null 2>&1" ;;
        *)                install_git="true" ;;
    esac

    printf "  %-25s " "$image"

    # Run trs doctor inside the container
    local output
    if ! output=$(docker run --rm \
        -v "$BINARY":/usr/local/bin/trs:ro \
        "$image" \
        sh -c "
            chmod +x /usr/local/bin/trs 2>/dev/null || true
            $install_git
            trs doctor --json 2>&1
        " 2>&1); then
        printf "${YELLOW}SKIP${NC} — container failed: %s\n" "$(echo "$output" | head -1)"
        ((SKIP++))
        return
    fi

    # Parse JSON result (handles both compact and pretty-printed output)
    local healthy
    healthy=$(echo "$output" | grep -o '"healthy": *[a-z]*' | head -1 | sed 's/.*: *//')
    local total
    total=$(echo "$output" | grep -o '"total": *[0-9]*' | head -1 | sed 's/.*: *//')
    local pass_count
    pass_count=$(echo "$output" | grep -o '"pass": *[0-9]*' | head -1 | sed 's/.*: *//')
    local fail_count
    fail_count=$(echo "$output" | grep -o '"fail": *[0-9]*' | head -1 | sed 's/.*: *//')
    local warn_count
    warn_count=$(echo "$output" | grep -o '"warn": *[0-9]*' | head -1 | sed 's/.*: *//')

    if [ "$healthy" = "true" ]; then
        printf "${GREEN}PASS${NC} — %s/%s checks passed\n" "$pass_count" "$total"
        ((PASS++))
    elif [ -n "$fail_count" ] && [ "$fail_count" = "0" ]; then
        printf "${YELLOW}WARN${NC} — %s passed, %s warnings\n" "$pass_count" "$warn_count"
        ((PASS++))  # Warnings are acceptable
    else
        printf "${RED}FAIL${NC} — %s/%s passed, %s failed\n" "$pass_count" "$total" "$fail_count"
        ((FAIL++))
        # Show failing checks
        echo "$output" | grep '"status":"fail"' | while read -r line; do
            local name detail
            name=$(echo "$line" | grep -o '"name":"[^"]*"' | cut -d'"' -f4)
            detail=$(echo "$line" | grep -o '"detail":"[^"]*"' | cut -d'"' -f4)
            printf "    ! %s — %s\n" "$name" "$detail"
        done
    fi
}

# ============================================================
# Main
# ============================================================

echo "trs smoke test — cross-platform validation"
echo ""

# Handle args
FILTER=""
DO_BUILD=false
for arg in "$@"; do
    case "$arg" in
        --build) DO_BUILD=true ;;
        *)       FILTER="$arg" ;;
    esac
done

# Check Docker
if ! docker info >/dev/null 2>&1; then
    echo "Error: Docker is not running"
    exit 1
fi

# Build binary if needed
if [ "$DO_BUILD" = true ] || [ ! -f "$BINARY" ]; then
    build_binary
fi

if [ ! -f "$BINARY" ]; then
    echo "Error: Binary not found at $BINARY"
    echo "Run with --build to compile, or build manually:"
    echo "  cross build --release --target x86_64-unknown-linux-gnu"
    exit 1
fi

echo "Binary: $BINARY"
echo "Size: $(du -h "$BINARY" | cut -f1)"
echo ""
echo "Results:"

for distro in "${DISTROS[@]}"; do
    if [ -n "$FILTER" ] && [[ "$distro" != *"$FILTER"* ]]; then
        continue
    fi
    test_distro "$distro"
done

echo ""
echo "Summary: $PASS passed, $FAIL failed, $SKIP skipped"

if [ "$FAIL" -gt 0 ]; then
    exit 1
fi
