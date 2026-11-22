## Why

The gate fails open when a non-Cargo suite cannot start on base. A base worktree has no
installed dependencies, so without a `setup` command vitest exits with
`Cannot find module 'vitest'` and writes no report. The gate counts every target test in that
suite as `red:compile`, so a test that already passes on the old code is accepted as proof.
Found in a trial migration of ellesmere.ai: `sj migrate` wrote no `setup` for its pnpm suites,
and a test green on base passed the gate as `red:compile`.

Kind: bug

## Current Behaviour

- A vitest suite with no `setup`, base worktree without `node_modules`: the base run exits 1
  with no report, every target test is recorded `red:compile`, and the gate passes.

## Expected Behaviour

- The same run is an environment error: the gate names the suite, prints the runner's last
  output lines, suggests declaring `setup` (or `red_on_base = "skip"` with a reason), and
  exits 2.

## Unchanged Behaviour

- A base run that writes a report but fails to load the test file (a symbol missing on base)
  is still `red:compile` (GATE-033).
- A suite with a working `setup` runs on base as before (GATE-031).
- Cargo suites: a compile failure on base is still `red:compile`.

## Impact

`crates/sj-core/src/gate.rs` (base runs of non-Cargo suites). No config or CLI changes.
