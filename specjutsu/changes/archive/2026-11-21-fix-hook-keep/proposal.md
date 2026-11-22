## Why

A `[keep]` scenario must pass on base and at head, so it never has a red run. The pre-edit hook (`sj hook claude pre-edit`, `sj hook check-edit`) blocks a source edit unless every scenario the active task names has a red ledger record, with no `[keep]` exemption, so a task naming a `[keep]` scenario can never be worked on in `guided` or `strict` mode. PR #60 fixed the same gap in `sj verify` and `sj task done` (LOOP-020); the hook was missed.

Kind: bug

## Current Behaviour

- The active task names AUTH-005, a `[keep]` scenario whose test passes and has no red run. Editing `apps/api/src/auth.ts` exits 2 with `write a failing test for AUTH-005 first (sj test AUTH-005)`.

## Expected Behaviour

- The same edit exits 0: a `[keep]` scenario needs no red run before source edits. A task naming AUTH-005 `[keep]` and AUTH-006 with no red run blocks with a reason for AUTH-006 only.

## Unchanged Behaviour

- A task naming AUTH-005 `[keep]` and AUTH-006, not `[keep]`, with no red run for AUTH-006, still blocks the edit with `write a failing test for AUTH-006 first (sj test AUTH-006)`.

## Impact

`sj-core::hook::decide`, using `spec::keep_ids`. No CLI or file format change.
