#!/usr/bin/env bash
# Write the spec badge and commit it to the orphan sj-badge branch when it changed (CI-013).
# Run from the repository root on a push to the default branch; needs push access to origin.
set -euo pipefail
branch=sj-badge
tmp=$(mktemp -d)
sj badge --out "$tmp/badge.json"

wt="$tmp/wt"
work="sj-badge-$$"
if git ls-remote --exit-code --heads origin "$branch" > /dev/null 2>&1; then
  git fetch -q origin "$branch"
  git worktree add -q -b "$work" "$wt" FETCH_HEAD
else
  git worktree add -q --detach "$wt"
  git -C "$wt" checkout -q --orphan "$work"
  git -C "$wt" rm -rfq --ignore-unmatch .
fi
cleanup() {
  git worktree remove --force "$wt" 2> /dev/null || true
  git branch -D "$work" > /dev/null 2>&1 || true
  rm -rf "$tmp"
}
trap cleanup EXIT

cp "$tmp/badge.json" "$wt/badge.json"
git -C "$wt" add badge.json
if git -C "$wt" diff --cached --quiet; then
  echo "badge unchanged"
  exit 0
fi
git -C "$wt" -c user.name="github-actions[bot]" \
  -c user.email="41898282+github-actions[bot]@users.noreply.github.com" \
  commit -qm "badge: $(sed -n 's/.*"message":"\([^"]*\)".*/\1/p' "$tmp/badge.json")"
git -C "$wt" push -q origin "HEAD:refs/heads/$branch"
echo "badge published to $branch"
