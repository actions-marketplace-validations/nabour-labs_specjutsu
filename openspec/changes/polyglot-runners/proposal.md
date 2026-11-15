## Why

`sj test` and `sj gate` can only run `cargo test`. Your projects need vitest, pytest and Playwright, run per package, from a pnpm/Turbo or uv workspace. The gate's red-on-base also assumes Rust's layout: it overlays files under `tests/` and builds with Cargo. TypeScript projects co-locate tests, and a base checkout of a JS or Python repo has no installed dependencies until something installs them.

## What Changes

- A runner adapter interface. A runner turns a suite plus a test selection into a command, and reads results from a **JUnit XML** report. Adapters: `vitest`, `jest`, `pytest`, `playwright`, plus a generic `junit` runner for any command that writes JUnit XML. `cargo` stays as it is.
- Each suite may declare `setup` (e.g. `pnpm install --frozen-lockfile`, `uv sync`), `command` overrides and `env`. Defaults come from the detected package manager.
- `sj test [SCOPE]` runs every suite in scope. Scope can be a scenario ID, a package path or a suite. Output is failures-only across runners, in the same format as today.
- Red-on-base overlays the files that match the changed suites' test globs, plus each suite's `support` globs (fixtures, helpers). It runs the suite's `setup` in the base worktree before the base run.
- Per-suite `red_on_base = "run" | "skip"`. `skip` records `unverifiable` with the reason, and fails the gate unless `--allow-unverifiable` is given. Playwright defaults to `run`.
- The gate runs head and base per suite. The unspecified-change rule uses each package's `sources` globs.

## Capabilities

### New Capabilities
None.

### Modified Capabilities
- `run`: suites, runner adapters, JUnit results and multi-suite scope. *Prerequisite:* `test-ledger` archived.
- `gate`: per-suite red-on-base with glob overlay, setup in the base worktree, and a red-on-base policy per suite.

## Impact

- `sj-runners`: `Runner` trait, JUnit XML parser (`quick-xml`), five adapters.
- `sj-core::gate`: overlay and base runs per suite. A base worktree install can take minutes, so `setup` output goes to a log and only failures are shown.
- Fixtures: extend `fixtures/polyglot/` with passing, failing, non-compiling and green-on-base tests per runner.
