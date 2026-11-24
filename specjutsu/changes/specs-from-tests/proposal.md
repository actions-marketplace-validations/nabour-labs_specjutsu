## Why

A project with a mature test suite and no specs can't adopt specjutsu without first writing a
scenario for every behaviour its tests already pin down. That is OpenSpec's most-requested open
issue, and the brownfield pilot hit the same wall: thousands of tests, none linked to a scenario,
and no way to start except by hand. Tools that do this today (cc-sdd, GSD, BMAD) have a model read
the code. specjutsu never calls a model, but the test scanner already finds every test,
including those that carry no scenario ID, so the skeleton of a spec can be drafted
deterministically and the agent fills in the meaning.

## What Changes

- New command `sj spec infer <path>... [--capability <name>] [--change <name>] [--write]`. It
  takes the untraced tests under the given paths (tests with no scenario ID in their name, title,
  `spec:` comment or marker) and drafts one `[keep]` scenario for each.
- Without `--write` it is a dry run: one line per test with its proposed ID and heading, then
  counts. It writes nothing.
- With `--write` it creates `<spec_root>/changes/<change>/` (default name `infer-<capability>`)
  with a proposal, tasks and an `## ADDED Requirements` delta for the capability, and adds a
  `spec: <ID>` comment above each test. No other byte of a test file changes.
- Headings are derived from the test name. WHEN/THEN steps hold clarify markers that point at
  the test's file and line, so `sj spec check` reports them until the agent or a person writes
  the real behaviour.
- Tests already linked, and skipped or todo tests, are left out and counted.
- The `sj-propose` skill gains a section on drafting specs from existing tests: run infer, then
  read each test and replace the markers.
- Docs: CLI reference, brownfield adoption guide, CHANGELOG, and the "planned" row in
  `docs/competitive-analysis.md`.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `authoring`: `sj spec infer`.
- `skills`: `sj-propose` covers specs drafted from tests.

## Impact

- `crates/sj-core/src/authoring/infer.rs` (new): selection, heading derivation, delta and
  comment writing.
- `crates/sj-core/src/trace.rs`: expose the tests that link to no ID (already scanned, today
  dropped).
- `crates/sj-core/src/ids.rs`: reuse prefix derivation and next-number allocation.
- `crates/sj-cli/src/main.rs`: the `spec infer` subcommand.
- `skills/sj-propose/SKILL.md` and the generated adapter copies.
- README, manual, CHANGELOG, `docs/competitive-analysis.md`.
- No new dependencies.
