#!/bin/sh
# trs installer — downloads the prebuilt binary for your platform.
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/dPeluChe/trs/main/scripts/install.sh | sh
#
# Options (env vars):
#   TRS_VERSION=v0.5.2  — pin a specific release (default: latest)
#   TRS_INSTALL_DIR=... — install location (default: $HOME/.trs/bin)
#   TRS_NO_MODIFY_PATH=1 — skip PATH shell-rc modification

set -eu

# ------------------------------------------------------------------
# Config
# ------------------------------------------------------------------

REPO="dPeluChe/trs"
INSTALL_DIR="${TRS_INSTALL_DIR:-$HOME/.trs/bin}"
BIN_NAME="trs"

# ------------------------------------------------------------------
# Colors (only when writing to a terminal)
# ------------------------------------------------------------------

if [ -t 1 ]; then
    C_RESET='\033[0m'
    C_BOLD='\033[1m'
    C_GREEN='\033[0;32m'
    C_YELLOW='\033[0;33m'
    C_RED='\033[0;31m'
    C_CYAN='\033[0;36m'
    C_GRAY='\033[0;90m'
else
    C_RESET=''; C_BOLD=''; C_GREEN=''; C_YELLOW=''; C_RED=''; C_CYAN=''; C_GRAY=''
fi

info()    { printf '%b▸%b %s\n' "$C_CYAN" "$C_RESET" "$*"; }
ok()      { printf '%b✓%b %s\n' "$C_GREEN" "$C_RESET" "$*"; }
warn()    { printf '%b!%b %s\n' "$C_YELLOW" "$C_RESET" "$*" >&2; }
error()   { printf '%b✗%b %s\n' "$C_RED" "$C_RESET" "$*" >&2; exit 1; }

# ------------------------------------------------------------------
# Detect platform + arch
# ------------------------------------------------------------------

detect_platform() {
    os=$(uname -s 2>/dev/null || echo unknown)
    arch=$(uname -m 2>/dev/null || echo unknown)

    case "$os" in
        Darwin) os_tag=darwin ;;
        Linux)  os_tag=linux ;;
        *) error "unsupported OS: $os (install via: cargo install tars-cli)" ;;
    esac

    case "$arch" in
        x86_64|amd64)       arch_tag=x64 ;;
        arm64|aarch64)      arch_tag=arm64 ;;
        *) error "unsupported arch: $arch" ;;
    esac

    echo "${os_tag}-${arch_tag}"
}

# ------------------------------------------------------------------
# Resolve version (latest via GitHub API, or pinned via env var)
# ------------------------------------------------------------------

resolve_version() {
    if [ -n "${TRS_VERSION:-}" ]; then
        echo "$TRS_VERSION"
        return
    fi
    # Fetch latest release tag. Avoid jq dependency.
    tag=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" 2>/dev/null \
        | grep -o '"tag_name": *"[^"]*"' \
        | head -1 \
        | sed -E 's/.*"tag_name": *"([^"]*)".*/\1/')
    [ -n "$tag" ] || error "could not resolve latest release (set TRS_VERSION=v0.5.2)"
    echo "$tag"
}

# ------------------------------------------------------------------
# Download + install
# ------------------------------------------------------------------

download() {
    url="$1"
    out="$2"
    if command -v curl >/dev/null 2>&1; then
        curl -fsSL "$url" -o "$out"
    elif command -v wget >/dev/null 2>&1; then
        wget -qO "$out" "$url"
    else
        error "neither curl nor wget is available"
    fi
}

# ------------------------------------------------------------------
# PATH shell-rc hint
# ------------------------------------------------------------------

already_in_path() {
    case ":$PATH:" in
        *":$INSTALL_DIR:"*) return 0 ;;
        *) return 1 ;;
    esac
}

shell_rc_for() {
    case "${SHELL:-}" in
        */zsh)  echo "$HOME/.zshrc" ;;
        */bash) echo "$HOME/.bashrc" ;;
        */fish) echo "$HOME/.config/fish/config.fish" ;;
        *)      echo "" ;;
    esac
}

append_path_instructions() {
    rc=$(shell_rc_for)
    export_line="export PATH=\"$INSTALL_DIR:\$PATH\""
    fish_line="fish_add_path $INSTALL_DIR"

    printf '\n'
    warn "$INSTALL_DIR is not in your PATH."
    printf '  Add this line to '
    if [ -n "$rc" ]; then
        printf '%b%s%b:\n' "$C_BOLD" "$rc" "$C_RESET"
    else
        printf 'your shell rc:\n'
    fi
    case "${SHELL:-}" in
        */fish) printf '    %s%s%s\n' "$C_CYAN" "$fish_line" "$C_RESET" ;;
        *)      printf '    %s%s%s\n' "$C_CYAN" "$export_line" "$C_RESET" ;;
    esac
    printf '\n  Then restart your shell or: %bsource %s%s\n\n' "$C_CYAN" "${rc:-<rc>}" "$C_RESET"
}

# ------------------------------------------------------------------
# Main
# ------------------------------------------------------------------

printf '\n%btrs installer%b\n' "$C_BOLD" "$C_RESET"
printf '%b%s%b\n\n' "$C_GRAY" "https://github.com/$REPO" "$C_RESET"

platform=$(detect_platform)
info "platform: $platform"

version=$(resolve_version)
info "version:  $version"

# Build asset name: trs-linux-x64, trs-darwin-arm64, etc.
asset="trs-${platform}"
url="https://github.com/${REPO}/releases/download/${version}/${asset}"

info "url:      $url"
info "install:  $INSTALL_DIR/$BIN_NAME"
printf '\n'

mkdir -p "$INSTALL_DIR"
tmp=$(mktemp "${TMPDIR:-/tmp}/trs-install.XXXXXX")
trap 'rm -f "$tmp"' EXIT

info "downloading..."
download "$url" "$tmp"
chmod +x "$tmp"

# Sanity check: run --version to verify binary works on this system
if ! "$tmp" --version >/dev/null 2>&1; then
    error "downloaded binary failed to run — architecture mismatch?"
fi

mv "$tmp" "$INSTALL_DIR/$BIN_NAME"
ok "installed $BIN_NAME $version to $INSTALL_DIR/$BIN_NAME"

# PATH check
if already_in_path; then
    ok "$INSTALL_DIR is already in PATH"
    printf '\nRun: %btrs --help%b\n' "$C_CYAN" "$C_RESET"
else
    if [ -z "${TRS_NO_MODIFY_PATH:-}" ]; then
        append_path_instructions
    fi
fi

printf '\n%bDone.%b Try: %btrs doctor%b\n\n' "$C_GREEN" "$C_RESET" "$C_CYAN" "$C_RESET"
