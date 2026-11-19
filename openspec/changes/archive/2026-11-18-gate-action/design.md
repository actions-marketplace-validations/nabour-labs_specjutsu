## Context

`sj gate` prints a summary line plus one line per reason, and `--json` prints the same data (GATE-018, GATE-019). `sj trace --json` knows every scenario and whether its tests are skipped (TRACE-012). Releases publish `sj-<target>.tar.gz` archives and a `SHA256SUMS` file, and `.github/install-sj.sh` already installs a verified Linux build for this repository's CI. What's missing is everything a GitHub user sees: an Action to `uses:`, a PR comment and a badge.

## Goals / Non-Goals

**Goals:**
- One `uses: nabour-labs/specjutsu@v0` step gates a pull request, comments on it and keeps the comment current.
- A README badge any repository can publish without a hosted service.
- The README sells the quality gate first and the CLI second.

**Non-Goals:**
- A hosted GitHub App or a dashboard service. Nothing in this change needs a server or our own credentials.
- GitLab, Bitbucket and Azure Pipelines. `--format markdown` and `sj badge` are CI-neutral, so they can follow.
- Inline review comments on lines. The one summary comment comes first.
- Running without specs (zero-spec mode). That comes with `audit-checks`, which adds an `audit` command to the action.

## Decisions

### 1. Composite action in this repository, not a JavaScript or Docker action
`action.yml` at the root, `runs: using: composite`, with bash steps that call scripts in `scripts/action/`. A Docker action only runs on Linux, and a JavaScript action needs a bundled `node_modules` and a second release process. The composite action reuses the release assets and works on Linux, macOS and Windows runners (bash is on all three).

### 2. The binary renders, the scripts only move bytes
All logic that decides what the comment says lives in `sj` (`--format markdown`, `sj badge`), where it is unit tested and works in any CI. The scripts only install, call `gh api` to find and upsert the comment, append to `$GITHUB_STEP_SUMMARY` and push the badge branch. Rejected: rendering markdown from `--json` with `jq` in bash. It's untestable on Windows developer machines and duplicates logic.

### 3. Comment upsert by marker
`gh api repos/{owner}/{repo}/issues/{n}/comments --paginate` finds the first comment whose body starts with `<!-- specjutsu:gate -->`. If there is one, it's PATCHed; otherwise a new one is POSTed. A 403 or 404 from the write turns into the read-only warning (CI-011), never a step failure.

### 4. Badge on an orphan branch, served by shields.io
The README uses `https://img.shields.io/endpoint?url=https://raw.githubusercontent.com/<owner>/<repo>/sj-badge/badge.json`. The action commits only on pushes to the default branch, using the workflow's `GITHUB_TOKEN` with `contents: write`. Rejected: GitHub Pages (many repositories already use Pages for docs) and gists (they need a personal token).

### 5. Badge counts "tested", not "proven"
"Proven" (red before green) can only be checked for the scenarios a change touches; a badge must describe the whole project at any commit. "Tested" means at least one mapped test that is not skipped, which `sj trace` already computes. The message says `tested` so it doesn't claim more than it checks.

### 6. Version pinning
The action's `version` input defaults to the `sj` version released with the action tag, so `@v0.4.0` always runs `sj 0.4.0`. A moving `@v0` tag points at the latest 0.x release.

## Risks / Trade-offs

- [The comment adds noise to every PR] → One comment, edited in place, and `comment: false` turns it off.
- [Forks can't comment with the default token] → The job summary is always written, and CI-011 makes that a warning. Teams that want comments on fork PRs can use `pull_request_target`; the README documents it and its risks.
- [A badge with a high number on a project whose tests are weak] → The badge says `tested`, not `proven` or `correct`. `audit-checks` adds the signal for weak tests.
- [The action can't be fully tested in `cargo test`] → The scripts' tests stub `gh` on `PATH`, and this repository runs the real action on its own PRs.

## Principles

- **Must work on Windows, macOS and Linux:** the action's scripts are bash. Windows runners have bash (Git for Windows), so the action works there. The script tests (CI-007..013) stub `gh` and need bash, so they run on Linux and macOS and are skipped on Windows. The rendering (CI-001..006) is in the binary and tested on all three.
