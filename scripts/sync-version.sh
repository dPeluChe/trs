#!/bin/bash
# Sync version from Cargo.toml to all npm package.json files.
# Usage: ./scripts/sync-version.sh [version]
# If no version given, reads from Cargo.toml.

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
