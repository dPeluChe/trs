#!/bin/bash
# Free the disk a release cycle leaves behind: trs's own build artifacts.
#
# Cargo never garbage-collects target/. Every edit leaves incremental caches,
# test binaries and debug-symbol objects of older hashes behind: two weeks of
# work was 22.5 GB, of which about 1 GB was third-party dependencies, the only
# slow part to rebuild. Rebuilding this crate and its ~96 test binaries from
# nothing takes ~15 s, so the default removes everything of ours and keeps the
# dependencies.
#
# Usage: scripts/post-release-clean.sh [options]
#   (none)          dry run: show what would go, after checking the release
#   --yes           do it
#   --if-over GB    during development: act only when target/ is over GB, and
#                   skip the release check (nothing was released)
#   --force         skip the release check
#   --deps          also remove compiled third-party dependencies (cargo clean)
#   --bench-cache   also remove the benchmark tool cache (rebuilt by setup.sh)
#
# After a release this runs only once the tag exists and the release workflow
# for it finished with "success". It never touches ~/.cargo (shared with other
# projects), ~/.trs (history and saved outputs), or git.

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

YES=0 FORCE=0 DEPS=0 BENCH=0 OVER=""
while [ $# -gt 0 ]; do
  case "$1" in
    --yes) YES=1 ;;
    --force) FORCE=1 ;;
    --deps) DEPS=1 ;;
    --bench-cache) BENCH=1 ;;
    --if-over)
      OVER="${2:?--if-over needs a size in GB}"
      shift
      ;;
    -h | --help)
      sed -n '2,25p' "$0"
      exit 0
      ;;
    *)
      echo "unknown option: $1 (see --help)" >&2
      exit 2
      ;;
  esac
  shift
done

fail() {
  echo "post-release-clean: $*" >&2
  exit 1
}

grep -q '^name = "trs-cli"' Cargo.toml 2>/dev/null || fail "run it from the trs repo"

kb() { du -sk "$1" 2>/dev/null | cut -f1; }
free_kb() { df -k . | awk 'NR==2 {print $4}'; }
gb() { awk -v k="$1" 'BEGIN {printf "%.1f GB", k / 1048576}'; }

if [ -n "$OVER" ]; then
  have=$(kb target || true)
  have=${have:-0}
  if awk -v k="$have" -v g="$OVER" 'BEGIN {exit !(k / 1048576 <= g)}'; then
    echo "target/ is $(gb "$have"), not over $OVER GB: nothing to do"
    exit 0
  fi
  echo "target/ is $(gb "$have"), over $OVER GB"
elif [ "$FORCE" -eq 0 ]; then
  VERSION=$(grep -m1 '^version' Cargo.toml | sed 's/.*"\(.*\)".*/\1/')
  TAG="v$VERSION"
  git rev-parse -q --verify "refs/tags/$TAG" >/dev/null ||
    fail "no tag $TAG: nothing was released from this version (use --force to clean anyway)"
  command -v gh >/dev/null || fail "gh not found, cannot confirm the release (use --force)"
  run=$(gh run list --workflow release.yml --branch "$TAG" --limit 1 \
    --json status,conclusion -q '.[0] | "\(.status) \(.conclusion)"' 2>/dev/null || true)
  [ "$run" = "completed success" ] ||
    fail "release $TAG is '${run:-not found}', not 'completed success': wait for it, or use --force"
  echo "release $TAG: $run"
fi

[ -d target ] || { echo "no target/ here: nothing to do"; exit 0; }

before_target=$(kb target)
before_free=$(free_kb)
echo "target/ is $(gb "$before_target"); $(gb "$before_free") free on this disk"
[ "$YES" -eq 1 ] || echo "dry run (add --yes to delete):"

DRY=()
[ "$YES" -eq 1 ] || DRY=(--dry-run)

if [ "$DEPS" -eq 1 ]; then
  echo "- everything in target/, dependencies included"
  cargo clean ${DRY[@]+"${DRY[@]}"}
else
  echo "- this crate's artifacts, dev and release (dependencies stay)"
  cargo clean -p trs-cli ${DRY[@]+"${DRY[@]}"}
  cargo clean -p trs-cli --release ${DRY[@]+"${DRY[@]}"}
fi

if [ "$YES" -eq 1 ]; then
  rm -rf target/test-trs-home target/tmp
else
  echo "- target/test-trs-home, target/tmp"
fi

if [ "$BENCH" -eq 1 ]; then
  BENCH_DIR="${TRS_BENCH_DIR:-$HOME/.cache/trs-bench}"
  echo "- benchmark cache $BENCH_DIR ($(gb "$(kb "$BENCH_DIR" || echo 0)"))"
  [ "$YES" -eq 1 ] && rm -rf "$BENCH_DIR"
fi

if [ "$YES" -eq 1 ]; then
  after_target=$(kb target 2>/dev/null || echo 0)
  after_free=$(free_kb)
  echo "target/ now $(gb "$after_target"); freed $(gb $((before_free < after_free ? after_free - before_free : 0)))"
fi
