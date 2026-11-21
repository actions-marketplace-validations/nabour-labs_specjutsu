## Why

`sj spec check` prints each `[NEEDS CLARIFICATION: …]` marker as a `clarify` line, but it strips
every inline-code span from the line before it looks for markers, so the code text inside a
question disappears. The question "do `PreToolUse` hooks fire for `apply_patch` file edits?"
prints as `clarify do  hooks fire for  file edits?`, which is unreadable for whoever resolves it.

## What Changes

- Clarify questions keep their inline-code spans. Markers that are themselves inside an inline-code
  span are still ignored.

## Capabilities

### New Capabilities

### Modified Capabilities

- `authoring`: `sj spec check` reports open questions with their inline code intact.

## Impact

`crates/sj-core/src/authoring/check.rs` and its tests. No CLI or format change.
