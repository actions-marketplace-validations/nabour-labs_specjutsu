#!/usr/bin/env bash
# Run the gate (or the audit), write the job summary, and create or update the PR comment
# (CI-008..012, CI-014).
# Env: SJ_COMMAND (gate|audit), SJ_BASE, SJ_PR, SJ_COMMENT (true|false), SJ_FAIL_ON
#      (gate|never), GITHUB_REPOSITORY, GITHUB_STEP_SUMMARY, GH_TOKEN.
set -uo pipefail
body=$(mktemp)

case "${SJ_COMMAND:-gate}" in
  gate)
    marker='<!-- specjutsu:gate -->'
    sj gate --base "$SJ_BASE" --format markdown > "$body"
    code=$? ;;
  audit)
    marker='<!-- specjutsu:audit -->'
    sj audit --base "$SJ_BASE" --registry --format markdown > "$body"
    code=$? ;;
  *)
    echo "::error::unknown command '$SJ_COMMAND' (expected gate or audit)"
    exit 2 ;;
esac

summary="${GITHUB_STEP_SUMMARY:-/dev/null}"
if [ "$code" -eq 2 ]; then
  echo "sj ${SJ_COMMAND:-gate} could not run (exit 2); see the step log." >> "$summary"
  exit 2
fi
cat "$body" >> "$summary"

if [ "${SJ_COMMENT:-true}" = true ] && [ -n "${SJ_PR:-}" ]; then
  repo="$GITHUB_REPOSITORY"
  id=$(gh api "repos/$repo/issues/$SJ_PR/comments" --paginate \
    --jq ".[] | select(.body | startswith(\"$marker\")) | .id" 2> /dev/null | head -n1)
  if [ -n "$id" ]; then
    gh api -X PATCH "repos/$repo/issues/comments/$id" -F "body=@$body" > /dev/null 2>&1
  else
    gh api -X POST "repos/$repo/issues/$SJ_PR/comments" -F "body=@$body" > /dev/null 2>&1
  fi || echo "::warning::cannot comment on this pull request (read-only token); see the job summary"
fi

if [ "${SJ_FAIL_ON:-gate}" = never ]; then
  exit 0
fi
exit "$code"
