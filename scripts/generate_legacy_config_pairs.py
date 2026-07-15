#!/usr/bin/env python3
"""Regenerate television/config/legacy_config_templates.json.

Extracts every version of .config/config.toml ever shipped (tv used to
auto-write it to users' config directories on first run) and flattens each
to leaf (path, value) pairs. `tv migrate-config` matches a user's config
against these templates to tell machine-written boilerplate apart from
deliberate settings.

Templates are kept separate (deduplicated, not unioned) on purpose: a value
only counts as boilerplate if it appears in the template the user's file
descends from — e.g. `ui_scale = 80` was the default of one era and a user
choice in any other.

The current template is excluded: it is never auto-written, so a user config
matching it was copied deliberately. The list is otherwise frozen — tv
stopped auto-writing the config file — so regeneration should only be needed
if this script itself changes.

Usage: python3 scripts/generate_legacy_config_pairs.py > television/config/legacy_config_templates.json
"""

import json
import subprocess
import sys
import tomllib


def git(*args, binary=False):
    out = subprocess.run(["git", *args], capture_output=True, check=True)
    return out.stdout if binary else out.stdout.decode("utf-8").strip()


current_blob = git("rev-parse", "HEAD:.config/config.toml")
commits = git("log", "--format=%H", "main", "--", ".config/config.toml").split()

blobs = []
for commit in commits:
    blob = subprocess.run(
        ["git", "rev-parse", f"{commit}:.config/config.toml"],
        capture_output=True,
        text=True,
    ).stdout.strip()
    if blob and blob != current_blob and blob not in blobs:
        blobs.append(blob)


def flatten(prefix, node, out):
    if isinstance(node, dict):
        for k, v in node.items():
            flatten(prefix + (k,), v, out)
    else:
        out.append((prefix, node))


templates = []
seen = set()
failures = 0

for blob in blobs:
    try:
        doc = tomllib.loads(git("cat-file", "blob", blob, binary=True).decode("utf-8"))
    except Exception as e:
        failures += 1
        print(f"skipping unparseable template {blob}: {e}", file=sys.stderr)
        continue
    leaves = []
    flatten((), doc, leaves)
    key = frozenset(
        (path, json.dumps(value, sort_keys=True)) for path, value in leaves
    )
    if key in seen:
        continue
    seen.add(key)
    templates.append(
        {"pairs": [{"path": list(path), "value": value} for path, value in leaves]}
    )

json.dump(templates, sys.stdout, separators=(",", ":"))
print()
print(
    f"{len(blobs) - failures}/{len(blobs)} historical templates parsed, "
    f"{len(templates)} distinct",
    file=sys.stderr,
)
