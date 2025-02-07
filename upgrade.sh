#!/usr/bin/env bash

set -eo pipefail

SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)
echo "$SCRIPT_DIR"
cd "$SCRIPT_DIR" || { echo "fatal error" >&2; exit 1; }
SQLITE3_LIB_DIR="$SCRIPT_DIR/sqlite3"

if [[ -z "$1" ]]
then
  echo "USAGE: ./upgrade.sh VERSION"
  echo "...where VERSION is the 7-digit sqlite version, like 3470200"
  exit 2
fi

set -u

SQLITE_VERSION="$1"
AMALGAMATION="sqlite-amalgamation-$SQLITE_VERSION"

# Download and extract amalgamation
curl -O "https://sqlite.org/$(date +%Y)/$AMALGAMATION.zip"
unzip -p "$AMALGAMATION.zip" "$AMALGAMATION/sqlite3.c" > "$SQLITE3_LIB_DIR/sqlite3.c"
unzip -p "$AMALGAMATION.zip" "$AMALGAMATION/sqlite3.h" > "$SQLITE3_LIB_DIR/sqlite3.h"
unzip -p "$AMALGAMATION.zip" "$AMALGAMATION/sqlite3ext.h" > "$SQLITE3_LIB_DIR/sqlite3ext.h"
rm -f "$AMALGAMATION.zip"

# update crate version
CRATE_VERSION="1.${SQLITE_VERSION:1}.0"
CRATE_SHORT_VERSION="1.${SQLITE_VERSION:1}"
sed -i -E "s/^version = \"1\.[0-9]+\.[0-9]+\"$/version = \"$CRATE_VERSION\"/" \
  $SCRIPT_DIR/Cargo.toml
sed -i -E "s/^libsqlite3-hotbundle = \"1.[0-9]+\"$/libsqlite3-hotbundle = \"$CRATE_SHORT_VERSION\"/" \
  $SCRIPT_DIR/README.md

# preview and confirm changes
git diff --stat sqlite3
git diff Cargo.toml README.md

read -p "Commit (y/n)? " choice
case "$choice" in
  y|Y ) echo "committing" ;;
  n|N ) exit 1 ;;
  * ) echo "invalid"; exit 1 ;;
esac

# commit and tag
git commit -am "sqlite3 upgrade to $SQLITE_VERSION"
git tag "v$CRATE_VERSION"
