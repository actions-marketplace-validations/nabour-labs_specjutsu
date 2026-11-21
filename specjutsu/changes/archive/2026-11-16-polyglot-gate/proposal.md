## Why

`sj gate` can only prove red-on-base with Cargo. Once `polyglot-runners` can run vitest, Jest, Playwright, pytest and JUnit suites, the gate has to use them: overlay a TypeScript project's co-located tests on the base checkout, install the base's dependencies before running there, and let an end-to-end suite that can't run on base say so instead of passing silently. The unspecified-change rule also still guesses what is source from a fixed list, when `specjutsu.toml` now says which files each package owns.

## What Changes

- Red-on-base runs per suite. The gate overlays every changed file that matches a suite's `tests` or `support` globs on the base worktree, runs the suite's `setup` there once, and runs only the target tests.
- New suite keys: `setup` (e.g. `pnpm install --frozen-lockfile`, `uv sync`), `support` (globs for fixtures and helpers to overlay), and `red_on_base = "run" | "skip"` with a `reason`. `skip` records `unverifiable` and fails the gate unless `--allow-unverifiable` is given.
- The gate runs head per suite, so mapped tests must pass at head in every runner.
- With packages declared, the unspecified-change rule uses each package's `sources` globs. A top-level `non_source` list extends the built-in non-source list (moved here from `project-config`, GATE-038).
- GATE-026 changes from "no root `Cargo.toml`" to "no suite configured": the error names the supported runners and suggests `sj adopt`.

## Capabilities

### New Capabilities
None.

### Modified Capabilities
- `gate`: per-suite red-on-base with glob overlay and base setup, a red-on-base policy per suite, source-glob classification, and the runner check.

## Impact

- `sj-core::gate`: overlay, setup and base runs per suite through the `polyglot-runners` adapters. A base install can take minutes, so `setup` output goes to a log and only failures are shown.
- `sj-core::config`: `setup`, `support`, `red_on_base`, `reason` and `non_source`.
- Depends on `polyglot-runners`.
