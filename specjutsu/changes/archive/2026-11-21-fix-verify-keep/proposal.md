## Why

A `[keep]` scenario must pass on base and at head, so it never has a red run, and the gate already counts a passing one as `green:keep` (GATE-039). `sj verify` and `sj task done` ask the ledger for red-before-green on every passing scenario, `[keep]` ones included, so a finished bug change always shows `unproven <ID> (no red run recorded)` for its `[keep]` scenarios and never reaches `verify: ok`. The open change `fix-base-run-no-report` hit it on GATE-047.

Kind: bug

## Current Behaviour

- A change adds AUTH-005 `[keep]` and AUTH-006, both with passing tests and no red run. `sj verify` prints `unproven AUTH-005 (no red run recorded)` and `unproven AUTH-006 (no red run recorded)`. `sj task done` for the task naming both warns about both.

## Expected Behaviour

- `sj verify` and `sj task done` report only `unproven AUTH-006`: a passing `[keep]` scenario needs no red run.

## Unchanged Behaviour

- A non-keep scenario with only passing runs is still unproven in `sj verify` and `sj task done` (AUTH-006 above).

## Impact

`sj-core::agent_loop::{verify, task}`, plus a shared `spec::keep_ids` used by the gate. No CLI or file format change.
