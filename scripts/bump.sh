#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd $SCRIPT_DIR/..

echo "Bumping version in workspace crates: $1"

TOML_FILES="$(git ls-files 'crates/television-*/Cargo.toml')"
perl -pi -e "s/^version .*= .*\$/version = \"$1\"/" -- $TOML_FILES
perl -pi -e "s/^(television-[a-z]+)\\s*=\\s*{.*\$/\\1 = { path = \"..\/\\1\", version = \"$1\" }/" -- $TOML_FILES

echo "Bumping root Cargo.toml dependencies to $1"

perl -pi -e "s/^(television-[a-z]+)\\s*=\\s*{.*\$/\\1 = { path = \"crates\/\\1\", version = \"$1\" }/" -- Cargo.toml

echo "Done"
