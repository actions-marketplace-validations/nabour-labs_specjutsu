#!/usr/bin/env bash
# Rebuild each case's expected/ by running the installed `openspec archive` on a copy of before/.
# Tests never call openspec; rerun this after an OpenSpec upgrade and review the diff.
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
for case_dir in "$here"/*/; do
  work="$(mktemp -d)"
  cp -R "$case_dir/before/." "$work/"
  change="$(ls "$work/openspec/changes" | grep -v '^archive$')"
  (cd "$work" && openspec archive "$change" --yes >/dev/null)
  rm -rf "$case_dir/expected"
  mkdir -p "$case_dir/expected"
  cp -R "$work/openspec/specs" "$case_dir/expected/specs"
  rm -rf "$work"
  echo "regenerated $(basename "$case_dir")"
done
