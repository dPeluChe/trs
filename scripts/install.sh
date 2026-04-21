#!/bin/sh
# trs installer — downloads the prebuilt binary for your platform.
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/dPeluChe/trs/main/scripts/install.sh | sh
#
# Options (env vars):
#   TRS_VERSION=v0.5.5  — pin a specific release (default: latest)
#   TRS_INSTALL_DIR=... — override install location
#
# Install dir selection (in priority order):
#   1. $TRS_INSTALL_DIR if set
#   2. $HOME/.local/bin if already in $PATH (XDG, ~80% of modern systems)
#   3. $HOME/bin if already in $PATH
#   4. $HOME/.local/bin as fallback (with a one-time PATH warning)

set -eu

# ------------------------------------------------------------------
# Config
# ------------------------------------------------------------------

REPO="dPeluChe/trs"
BIN_NAME="trs"

# INSTALL_DIR is resolved later by pick_install_dir so we can check $PATH.

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
        *) error "unsupported OS: $os (install via: cargo install trs-cli)" ;;
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

path_contains() {
    case ":$PATH:" in
        *":$1:"*) return 0 ;;
        *) return 1 ;;
    esac
}

already_in_path() {
    path_contains "$INSTALL_DIR"
}

# Pick the best install dir: prefer one that's already in $PATH so new installs
# work with zero shell-config changes. Falls back to $HOME/.local/bin (XDG).
# Writes the chosen path to stdout; caller assigns to INSTALL_DIR.
pick_install_dir() {
    if [ -n "${TRS_INSTALL_DIR:-}" ]; then
        echo "$TRS_INSTALL_DIR"
        return
    fi
    for d in "$HOME/.local/bin" "$HOME/bin"; do
        if path_contains "$d"; then
            echo "$d"
            return
        fi
    done
    # Nothing in PATH yet — default to XDG convention so the user only has to
    # add it to PATH once and it'll be correct for every future tool too.
    echo "$HOME/.local/bin"
}

shell_rc_for() {
    # Prefer the rc file that's read by ALL shell invocations — interactive,
    # login, and non-interactive subshells (the kind IDE agents like
    # Antigravity / Windsurf / Cursor spawn). Otherwise trs works in your
    # terminal but `command not found` in IDE-spawned shells.
    #
    #   zsh  — .zshenv     (every invocation)   vs .zshrc (interactive only)
    #   bash — .bashrc     (interactive non-login; also sourced by many IDEs)
    #          (.profile covers the non-interactive case but is widely ignored)
    #   fish — config.fish (fish reads this for both interactive and scripts)
    case "${SHELL:-}" in
        */zsh)  echo "$HOME/.zshenv" ;;
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
        */fish) printf '    %b%s%b\n' "$C_CYAN" "$fish_line" "$C_RESET" ;;
        *)      printf '    %b%s%b\n' "$C_CYAN" "$export_line" "$C_RESET" ;;
    esac
    printf '\n  Then restart your shell or: %bsource %s%b\n\n' "$C_CYAN" "${rc:-<rc>}" "$C_RESET"
}

# ------------------------------------------------------------------
# Detect existing trs install (from npm / brew / cargo) and warn
# ------------------------------------------------------------------

check_existing_install() {
    existing=$(command -v trs 2>/dev/null || true)
    if [ -z "$existing" ]; then
        return 0
    fi
    target="$INSTALL_DIR/$BIN_NAME"
    # Resolve symlinks to compare real paths
    existing_real=$(cd "$(dirname "$existing")" 2>/dev/null && pwd -P)/$(basename "$existing")
    target_dir_real=$(mkdir -p "$INSTALL_DIR" && cd "$INSTALL_DIR" && pwd -P)
    target_real="$target_dir_real/$BIN_NAME"
    if [ "$existing_real" = "$target_real" ]; then
        return 0
    fi
    # Detect source based on path heuristic
    source_hint="unknown source"
    case "$existing" in
        */node_modules/*|*/npm/*|/opt/homebrew/bin/trs|/usr/local/bin/trs)
            if [ -f "$HOME/.npmrc" ] || command -v npm >/dev/null 2>&1; then
                source_hint="probably npm (try: npm uninstall -g @dpeluche/trs)"
            else
                source_hint="from Homebrew or system package manager"
            fi
            ;;
        *"/.cargo/bin/"*) source_hint="from cargo (try: cargo uninstall trs-cli)" ;;
    esac
    printf '\n'
    warn "Another trs is already installed at:"
    printf '       %s\n' "$existing"
    printf '       (%s)\n\n' "$source_hint"
    printf '  After this install, PATH order decides which runs.\n'
    printf '  Put %b%s%b first in PATH to prefer this new install.\n\n' \
        "$C_CYAN" "$INSTALL_DIR" "$C_RESET"
}

# ------------------------------------------------------------------
# Main
# ------------------------------------------------------------------

printf '\n%btrs installer%b\n' "$C_BOLD" "$C_RESET"
printf '%b%s%b\n\n' "$C_GRAY" "https://github.com/$REPO" "$C_RESET"

INSTALL_DIR=$(pick_install_dir)

platform=$(detect_platform)
info "platform: $platform"

check_existing_install

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

# PATH check — with pick_install_dir's zero-config default, this should
# rarely fire. When it does, print the exact line the user needs to add.
if already_in_path; then
    ok "$INSTALL_DIR is in PATH"
    printf '\n%bDone.%b Run: %btrs doctor%b\n\n' "$C_GREEN" "$C_RESET" "$C_CYAN" "$C_RESET"
else
    append_path_instructions
    printf '%bDone.%b After reloading your shell, try: %btrs doctor%b\n\n' \
        "$C_GREEN" "$C_RESET" "$C_CYAN" "$C_RESET"
fi
