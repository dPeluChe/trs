#!/bin/bash
# Sync version from Cargo.toml to all npm package.json files.
# Usage: ./scripts/sync-version.sh [version]
# If no version given, reads from Cargo.toml.
#
# Not part of a release: release.yml does this from the tag when it publishes,
# and the committed npm versions are left as they are. Run it only to try the
# npm packaging locally, and do not commit what it changes.

set -e

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"

if [ -n "$1" ]; then
  VERSION="$1"
else
  VERSION=$(grep '^version' "$REPO_ROOT/Cargo.toml" | head -1 | sed 's/.*"\(.*\)".*/\1/')
fi

echo "Syncing version: $VERSION"

# Base package
node -e "
  const pkg = require('$REPO_ROOT/npm/package.json');
  pkg.version = '$VERSION';
  for (const dep in pkg.optionalDependencies) {
    pkg.optionalDependencies[dep] = '$VERSION';
  }
  require('fs').writeFileSync('$REPO_ROOT/npm/package.json', JSON.stringify(pkg, null, 2) + '\n');
"

# Platform packages
for dir in "$REPO_ROOT"/npm/platforms/*/; do
  node -e "
    const pkg = require('${dir}package.json');
    pkg.version = '$VERSION';
    require('fs').writeFileSync('${dir}package.json', JSON.stringify(pkg, null, 2) + '\n');
  "
done

echo "Done. All packages at v$VERSION"
echo
echo "npm/ now differs from git; do not commit it (release.yml sets these from the tag)."
echo "To release v$VERSION: merge the Cargo.toml bump, then tag and clean up:"
echo "  git tag -a v$VERSION -m v$VERSION && git push origin v$VERSION"
echo "  scripts/post-release-clean.sh --wait --yes"
