#!/bin/bash
set -euo pipefail

# get new version
NEW_VERSION=$(python scripts/get_version.py)

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR/.."

echo "Bumping version in workspace crates: $NEW_VERSION"

TOML_FILES="$(git ls-files 'crates/television-*/Cargo.toml')"
perl -pi -e "s/^version .*= .*\$/version = \"$NEW_VERSION\"/" -- $TOML_FILES
perl -pi -e "s/^(television-[a-z]+)\\s*=\\s*{.*\$/\\1 = { path = \"..\/\\1\", version = \"$NEW_VERSION\" }/" -- $TOML_FILES

echo "Bumping root Cargo.toml dependencies to $NEW_VERSION"

perl -pi -e "s/^(television-[a-z]+)\\s*=\\s*{.*\$/\\1 = { path = \"crates\/\\1\", version = \"$NEW_VERSION\" }/" -- Cargo.toml

echo "Done"
