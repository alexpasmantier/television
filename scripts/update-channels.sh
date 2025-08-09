#!/usr/bin/env bash
set -euo pipefail

owner="alexpasmantier"
repo="television"
root="cable/unix"
ref="main"
dest="${XDG_CONFIG_HOME:-$HOME/.config}/television"

mkdir -p "$dest"

auth_header=()
[[ -n "${GITHUB_TOKEN:-}" ]] && auth_header=(-H "Authorization: Bearer $GITHUB_TOKEN")

curl -sS -L "${auth_header[@]}" \
  "https://api.github.com/repos/$owner/$repo/git/trees/$ref?recursive=1" \
| jq -r --arg root "$root/" '
    .tree[]
    | select(.type=="blob" and (.path | startswith($root)))
    | .path
' \
| while IFS= read -r relpath; do
    echo "Downloading $relpath"
    mkdir -p "$dest/$(dirname "$relpath")"
    curl -sS -L --fail \
      "https://raw.githubusercontent.com/$owner/$repo/$ref/$relpath" \
      -o "$dest/$relpath"
  done

echo "Done -> $dest"

