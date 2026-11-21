## Why

Agents and people run `cargo test` and then read through hundreds of `ok` lines to find the one that failed. That wastes tokens and attention. Nothing records the runs either, so the red-before-green claim exists only in commit messages. v0.3's agent loop (`status`, `next`, strict mode) needs two things first: a test command an agent can call cheaply, and a local record of what each run proved.

## What Changes

- New command `sj test [SCOPE]` runs `cargo test` and prints failures only: a one-line summary, then each failing test with the scenario IDs it maps to and a short excerpt of its failure output. If the build fails, it prints the compiler's error lines instead.
- `SCOPE` narrows the run. A scenario ID (`GATE-004`) runs only the tests mapped to that scenario. Anything else is passed to cargo as a package name (`-p`). With no scope, the whole workspace runs.
- Exit codes match `sj gate`: 0 when every test that ran passed, 1 for any test failure or build failure, 2 when sj itself errors.
- `--json` prints the same result in machine-readable form.
- Every `sj test` run appends records to a local evidence ledger in `.specjutsu/ledger/`. Each record holds: run ID, timestamp, git tree hash of the working tree, test path, mapped scenario IDs and result. The results are passed, failed, ignored and build-failed.
- New command `sj ledger [ID] [--json]` reads the ledger and reports, per scenario: the latest result, and whether a red run was recorded before a green one on a different tree.
- `--no-ledger` turns recording off for a run.

## Capabilities

### New Capabilities
- `run`: running a project's tests through sj. This covers scope selection, failures-only output, exit codes and JSON output. Later runners (pytest, vitest, …) add requirements here.
- `ledger`: the local evidence ledger. This covers what a run records, where and in what format, and how recorded history is queried to prove red-before-green.

### Modified Capabilities
None. `sj gate` keeps proving red-on-base by running tests in CI. Making the gate or strict mode read the ledger is a later change.

## Impact

- `crates/sj-runners/src/cargo.rs`: `cargo_test` must also return failure excerpts and say whether the build failed. Today a compile failure looks the same as an empty run.
- `crates/sj-core`: new `ledger` module (record type, append, load, red→green query) and a helper that hashes the working tree without touching the user's index.
- `crates/sj-cli`: new `test` and `ledger` subcommands.
- No new dependencies. Timestamps are Unix milliseconds from `std::time`, and records are JSON Lines through serde_json.
- `.specjutsu/ledger/` is already in `.gitignore`, so the ledger stays local and is never committed.
