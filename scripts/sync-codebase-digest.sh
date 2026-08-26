#!/usr/bin/env bash
# Regenerate docs/development/codebase-digest.md with a fresh `trs ingest`.
#
# Run this before tagging a release, or any time src/ has moved
# significantly since the last digest. The committed digest is
# meant to be an up-to-date, drop-in context primer for any AI
# agent working on this repo: if it drifts too far from HEAD,
# the digest loses its point.
#
# Usage:
#   ./scripts/sync-codebase-digest.sh
#
# Requires: the local trs binary in PATH. If you don't have it
# installed, build from source first:
#   cargo build --release
#   ./target/release/trs ingest --budget 128k -o docs/development/codebase-digest.md

set -euo pipefail

DIGEST="docs/development/codebase-digest.md"

if ! command -v trs >/dev/null 2>&1; then
  # Fall back to the release build inside the repo, if present.
  if [[ -x "./target/release/trs" ]]; then
    TRS="./target/release/trs"
  else
    echo "trs not in PATH and no ./target/release/trs build found." >&2
    echo "Build first with: cargo build --release" >&2
    exit 1
  fi
else
  TRS="trs"
fi

"$TRS" ingest --budget 128k -o "$DIGEST"

if git diff --quiet -- "$DIGEST"; then
  echo "codebase-digest is already up to date."
else
  echo "codebase-digest regenerated. Diff follows:"
  git --no-pager diff --stat -- "$DIGEST"
  echo
  echo "Commit the change if the update looks right."
fi
