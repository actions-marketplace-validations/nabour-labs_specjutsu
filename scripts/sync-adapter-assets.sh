#!/usr/bin/env bash
# Copy skills/ and adapters/claude/ into crates/sj-adapters/assets/, which the crate embeds.
# The repository-root files are the source; edit those, then run this.
set -euo pipefail
cd "$(dirname "$0")/.."
assets=crates/sj-adapters/assets
rm -rf "$assets"
mkdir -p "$assets/claude"
cp adapters/claude/CLAUDE.block.md "$assets/claude/"
for d in skills/sj-*/; do
  mkdir -p "$assets/$d"
  cp "$d/SKILL.md" "$assets/$d"
done
echo "synced $assets"
