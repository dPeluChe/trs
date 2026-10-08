#!/bin/bash
# Remove trs's own build output from target/ and keep the compiled
# dependencies (the slow part to rebuild). Cargo never removes old artifacts.
#
# Usage: scripts/post-release-clean.sh [options]
#   (none)          dry run: show what would go, after checking the release
#   --yes           do it
#   --if-over GB    during development: act only when target/ is over GB, and
#                   skip the release check
#   --wait          wait for the release run of the tag to finish (up to 30 min,
#                   or 3 min if it has not appeared yet), then clean: the last
#                   command after pushing the tag
#   --force         skip the release check
#   --deps          also remove compiled dependencies (a full cargo clean)
#   --bench-cache   also remove the benchmark tool cache (rebuilt by setup.sh)
#
# After a release it runs only once the tag exists and the release workflow
# for it finished with "success". It never touches ~/.cargo, ~/.trs or git.

set -euo pipefail
cd "$(dirname "$0")/.."

YES=0 FORCE=0 DEPS=0 BENCH=0 WAIT=0 OVER=""
POLL="${POST_RELEASE_POLL_SECONDS:-15}"
WAIT_MAX="${POST_RELEASE_WAIT_SECONDS:-1800}"
while [ $# -gt 0 ]; do
  case "$1" in
    --yes) YES=1 ;;
    --force) FORCE=1 ;;
    --wait) WAIT=1 ;;
    --deps) DEPS=1 ;;
    --bench-cache) BENCH=1 ;;
    --if-over)
      OVER="${2:?--if-over needs a size in GB}"
      shift
      ;;
    -h | --help)
      sed -n '2,/^$/p' "$0"
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

# Size in KB; 0 when the path is missing.
kb() { { du -sk "$1" 2>/dev/null || true; } | awk '{s += $1} END {print s + 0}'; }
gb() { awk -v k="$1" 'BEGIN {printf "%.1f GB", k / 1048576}'; }

grep -q '^name = "trs-cli"' Cargo.toml 2>/dev/null || fail "run it from the trs repo"
[ -d target ] || { echo "no target/ here: nothing to do"; exit 0; }

before=$(kb target)
if [ -n "$OVER" ]; then
  if awk -v k="$before" -v g="$OVER" 'BEGIN {exit !(k / 1048576 <= g)}'; then
    echo "target/ is $(gb "$before"), not over $OVER GB: nothing to do"
    exit 0
  fi
elif [ "$FORCE" -eq 0 ]; then
  TAG="v$(grep -m1 '^version' Cargo.toml | sed 's/.*"\(.*\)".*/\1/')"
  git rev-parse -q --verify "refs/tags/$TAG" >/dev/null ||
    fail "no tag $TAG: nothing was released from this version (use --force to clean anyway)"
  command -v gh >/dev/null || fail "gh not found, cannot confirm the release (use --force)"
  [ "$POLL" -ge 1 ] 2>/dev/null || fail "POST_RELEASE_POLL_SECONDS must be a whole number of seconds, 1 or more"
  waited=0
  while :; do
    run=$(gh run list --workflow release.yml --branch "$TAG" --limit 1 \
      --json status,conclusion -q '.[0] | "\(.status) \(.conclusion)"' 2>/dev/null) ||
      fail "gh could not read the release runs (logged in? use --force)"
    [ "$run" = "null null" ] && run="" # what gh prints before the run exists
    [ "$run" = "completed success" ] && break
    case "$run" in
      completed*) fail "release $TAG finished as '$run', not 'completed success'" ;;
    esac
    [ "$WAIT" -eq 1 ] ||
      fail "release $TAG is '${run:-no run found}', not 'completed success': pass --wait to wait for it, or --force"
    limit=$WAIT_MAX
    [ -n "$run" ] || limit=$((WAIT_MAX < 180 ? WAIT_MAX : 180))
    if [ "$waited" -ge "$limit" ]; then
      [ -n "$run" ] || fail "no release run for $TAG after ${waited}s: was the tag pushed?"
      fail "release $TAG is still '$run' after ${waited}s"
    fi
    echo "release $TAG: ${run:-no run yet}, checking again in ${POLL}s"
    sleep "$POLL"
    waited=$((waited + POLL))
  done
  echo "release $TAG: $run"
fi

echo "target/ is $(gb "$before")"
DRY=--dry-run
if [ "$YES" -eq 1 ]; then DRY=; else echo "dry run (add --yes to delete):"; fi

if [ "$DEPS" -eq 1 ]; then
  echo "- everything in target/, dependencies included"
  cargo clean $DRY
else
  echo "- this crate's artifacts, dev and release (dependencies stay)"
  cargo clean -p trs-cli $DRY
  cargo clean -p trs-cli --release $DRY
fi

echo "- target/test-trs-home, target/tmp"
[ "$YES" -eq 1 ] && rm -rf target/test-trs-home target/tmp

if [ "$BENCH" -eq 1 ]; then
  BENCH_DIR="${TRS_BENCH_DIR:-$HOME/.cache/trs-bench}"
  echo "- benchmark cache $BENCH_DIR"
  [ "$YES" -eq 1 ] && rm -rf "$BENCH_DIR"
fi

[ "$YES" -eq 0 ] || echo "target/ now $(gb "$(kb target)"), was $(gb "$before")"
exit 0
