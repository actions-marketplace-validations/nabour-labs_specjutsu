## Why

A bug fix has two jobs: make the broken behaviour work, and leave everything next to it
alone. Today specjutsu proves the first one (the reproduction test must be red on base) but
has no way to say "this must not change". Kiro's bugfix specs and Spec Kit's bug flow both
write down current, expected and unchanged behaviour, but they check it with model judgement.
specjutsu can check it with the same deterministic gate it already runs: a test for unchanged
behaviour must pass on base and at head.

## What Changes

- A scenario can be marked `[keep]` in its heading. A keep scenario documents behaviour the
  change must not alter.
- The gate requires a mapped test for every keep scenario, and that test must pass on base and
  at head. It is recorded as `green:keep`, and failing on base is an error, the opposite of a
  normal scenario.
- `sj spec new <name> --bug` scaffolds a proposal with Current, Expected and Unchanged sections.
- `sj spec check` warns when a `--bug` change has no keep scenario or no reproduction scenario.

## Capabilities

### New Capabilities

### Modified Capabilities

- `gate`: keep scenarios and the `green:keep` evidence.
- `authoring`: `[keep]` headings, `sj spec new --bug`, bugfix checks.

## Impact

`sj-core` gate classification and evidence ledger, spec parser, `sj spec new` and
`sj spec check`, the PR comment, the `sj-debug` skill and the README.
