## Why

specjutsu's gate already refuses unproven work, but only for someone who installs the binary, writes a workflow and reads a CI log. Developers who want to stop their AI agent from shipping nonsense look for a GitHub Action, not a CLI. The tools that spread through GitHub (Codecov, Coveralls, Semgrep) all do three things: one line installs them in a workflow, they comment on every PR, and they give a badge for the README. Each PR comment and each badge is seen by developers who don't use the tool yet. specjutsu has none of the three, so it has no way to spread.

This change makes the gate a GitHub Action with a PR comment and a spec coverage badge, and repositions the README around it: "the quality gate for AI-generated code".

## What Changes

- `sj gate --format markdown` prints the gate result as a PR comment body: a summary line, a table of scenarios with their evidence (`red:assert`, `red:compile`, `untested`, `green-on-base`, …) and a fix hint per failure. It carries a hidden marker so the comment can be found and updated.
- `sj badge` writes a shields.io endpoint JSON file: tested scenarios out of all scenarios, coloured by threshold.
- A composite action, `action.yml` at the repository root (`uses: nabour-labs/specjutsu@v0`). It installs a pinned `sj` with checksum verification and runs the gate against the PR base. It creates or updates one PR comment, writes the job summary, and on pushes to the default branch publishes the badge JSON to an `sj-badge` branch.
- README and site lead with the Action and the quality-gate positioning; the CLI stays documented as the engine underneath.

## Capabilities

### New Capabilities
- `ci`: the gate's PR-facing output (markdown comment, badge) and the GitHub Action that installs and runs it.

### Modified Capabilities
None. The gate's checks and exit codes are unchanged; `--format markdown` is a new output beside `--json`.

## Impact

- `sj-core`: a markdown renderer over the existing gate result, and the badge computation over the trace report.
- `sj-cli`: `sj gate --format markdown`, `sj badge [--out <path>]`.
- New `action.yml` and `scripts/action/` (install, comment upsert, badge publish). They use `gh`, which GitHub-hosted runners provide.
- `.github/workflows/ci.yml` runs the action on this repository's own PRs (dogfooding).
- README, site and manual: new positioning and a CI quickstart.
- `openspec/principles.md`: adds the `CI` prefix.
