#!/usr/bin/env bash
set -euo pipefail

case "$OSTYPE" in
  linux-gnu* | darwin*) OS="unix" ;;
  msys* | cygwin*) OS="windows" ;;
  *) echo "Unsupported OS: $OSTYPE" && exit 1 ;;
esac

owner="alexpasmantier"
repo="television"
root="cable/$OS"
ref="main"
dest="${XDG_CONFIG_HOME:-$HOME/.config}/television"

mkdir -p "$dest"

case " $* " in
*" --force "*) force_update=true ;;
*) force_update=false ;;
esac

curl -sS -L ${GITHUB_TOKEN:+-H "Authorization: Bearer $GITHUB_TOKEN"} \
  "https://api.github.com/repos/$owner/$repo/git/trees/$ref?recursive=1" \
| jq -r --arg root "$root/" '
    .tree[]
    | select(.type=="blob" and (.path | startswith($root)))
    | .path
' \
| while IFS= read -r relpath; do
    channel_name="$(basename "${relpath%.*}")"
    target="$dest/$relpath"
    if [ -e "$target" ] && ! $force_update; then
        echo "Channel $channel_name already exists at $target, SKIPPING..."
        continue
    fi
    mkdir -p "$dest/$root"
    curl -sS -L --fail \
      "https://raw.githubusercontent.com/$owner/$repo/$ref/$relpath" \
      -o "$target"
    echo "Saved $channel_name to $target"
  done

echo "Done -> $dest"
