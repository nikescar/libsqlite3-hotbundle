#!/usr/bin/env bash

set -xeo pipefail

SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)
echo "$SCRIPT_DIR"
cd "$SCRIPT_DIR" || { echo "fatal error" >&2; exit 1; }
SQLITE3MC_TEMP_DIR=$(mktemp -d)

if [[ -z "$1" ]]
then
  echo "USAGE: ./upgrade.sh TAG"
  echo "...where TAG is the SQLite3MultipleCiphers git tag, like v2.3.2"
  echo "Alternatively, use: ./upgrade.sh latest"
  exit 2
fi

set -u

SQLITE3MC_TAG="$1"

# Clone SQLite3MultipleCiphers to temporary directory
echo "Cloning SQLite3MultipleCiphers to temporary directory..."
git clone https://github.com/utelle/SQLite3MultipleCiphers.git "$SQLITE3MC_TEMP_DIR"
cd "$SQLITE3MC_TEMP_DIR" || { echo "Failed to create temporary directory" >&2; exit 1; }

# Fetch latest tags
git fetch --tags

if [[ "$SQLITE3MC_TAG" == "latest" ]]; then
  # Get the latest tag
  SQLITE3MC_TAG=$(git describe --tags "$(git rev-list --tags --max-count=1)")
  echo "Using latest tag: $SQLITE3MC_TAG"
fi

# Checkout the specified tag
git checkout "$SQLITE3MC_TAG"

# Get the SQLite version from the header
SQLITE_VERSION=$(grep "SQLITE_VERSION " src/sqlite3.h | head -1 | sed -E 's/.*"([0-9]+\.[0-9]+\.[0-9]+)".*/\1/')
echo "SQLite version: $SQLITE_VERSION"

# Get SQLite3MC version
SQLITE3MC_VERSION=$(grep "SQLITE3MC_VERSION_STRING" src/sqlite3mc_version.h | sed -E 's/.*"SQLite3 Multiple Ciphers ([0-9]+\.[0-9]+\.[0-9]+)".*/\1/')
echo "SQLite3MultipleCiphers version: $SQLITE3MC_VERSION"

# Convert SQLite version to 7-digit format (e.g., 3.51.3 -> 3510300)
SQLITE_VERSION_NUM=$(echo "$SQLITE_VERSION" | awk -F. '{printf "%d%02d%02d00", $1, $2, $3}')

# Copy source files to main project
echo "Copying source files to project..."
rm -rf "$SCRIPT_DIR/sqlite3/aegis" "$SCRIPT_DIR/sqlite3/argon2" "$SCRIPT_DIR/sqlite3/ascon"
cp -r "$SQLITE3MC_TEMP_DIR/src/"* "$SCRIPT_DIR/sqlite3/"

# Return to script directory
cd "$SCRIPT_DIR"

# Cleanup temporary directory
rm -rf "$SQLITE3MC_TEMP_DIR"

# Update crate version based on SQLite version
CRATE_VERSION="1.${SQLITE_VERSION_NUM:1}.0+mc${SQLITE3MC_VERSION}"
CRATE_SHORT_VERSION="1.${SQLITE_VERSION_NUM:1}"
sed -i -E "s/^version = \"[^\"]+\"$/version = \"$CRATE_VERSION\"/" \
  "$SCRIPT_DIR/Cargo.toml"

if [[ -f "$SCRIPT_DIR/README.md" ]]; then
  sed -i -E "s/^libsqlite3-hotbundle = \"[^\"]+\"$/libsqlite3-hotbundle = \"$CRATE_SHORT_VERSION\"/" \
    "$SCRIPT_DIR/README.md"
fi

# Preview changes
echo "=== Changes summary ==="
echo "SQLite3MultipleCiphers tag: $SQLITE3MC_TAG"
echo "SQLite version: $SQLITE_VERSION ($SQLITE_VERSION_NUM)"
echo "SQLite3MC version: $SQLITE3MC_VERSION"
echo "Crate version: $CRATE_VERSION"
echo ""
git diff Cargo.toml README.md

read -p "Commit (y/n)? " choice
case "$choice" in
  y|Y ) echo "committing" ;;
  n|N ) exit 1 ;;
  * ) echo "invalid"; exit 1 ;;
esac

# Commit and tag
git add sqlite3/
git commit -am "sqlite3mc upgrade to $SQLITE3MC_TAG (SQLite $SQLITE_VERSION)"
git tag "v$CRATE_VERSION"
