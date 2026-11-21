## Why

Most code that needs specs already exists, and its behaviour is already written down in
the one place that is checked: its tests. Every SDD tool asks for specs to be written by
hand or reverse-engineered by a model, and "generate specs from existing code" is the
most-requested open feature in OpenSpec (#724). specjutsu can do the deterministic half
without a model: list every test that has no scenario ID, turn each into a draft scenario
with an ID, and pin it as `[keep]` behaviour so the gate proves it keeps passing. The agent
then fills in the WHEN and THEN lines, with the test as the source of truth.

## What Changes

- New `sj spec infer [<path>]`: previews a change that has one draft keep scenario per
  untraced test, grouped by test file, with IDs allocated. `--write` writes it.
- Draft steps are `[NEEDS CLARIFICATION]` markers, so `sj spec check` and `sj-clarify`
  route them to the agent or the user until they are written.
- `sj spec infer` prints a test-to-ID mapping and, with `--link`, renames or annotates each
  test using the convention its runner already uses for trace.
- New `sj-onboard` skill drives the loop for an existing codebase: `sj adopt`, `sj audit`,
  `sj spec infer`, fill the drafts from the test code, `sj gate`.

## Capabilities

### New Capabilities

### Modified Capabilities

- `authoring`: `sj spec infer`.
- `skills`: `sj-onboard`.

## Impact

New `sj spec infer` subcommand in `sj-cli` and inference in `sj-core::spec`, reusing trace's
test discovery. New `skills/sj-onboard`. Depends on `bugfix-changes` for `[keep]` scenarios.
