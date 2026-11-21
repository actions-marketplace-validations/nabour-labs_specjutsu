## Why

An example-based test proves one input. Many requirements are rules over all inputs: a
refund never exceeds the charge, a parser round-trips every valid document. Kiro turns such
requirements into property-based tests, but the link is IDE-only and optional, and nothing
checks it in CI. specjutsu can make it a gate rule: a scenario written as a property must be
proven by a property-based test, that test must fail on base, and the shrunk counterexample
goes into the evidence.

## What Changes

- A scenario can state a property with a `**FOR ALL**` step before its THEN step.
- `sj trace` checks that a property scenario's test is a property-based test: `proptest!` or
  `quickcheck` in Rust, `fc.assert` in TypeScript and JavaScript, `@given` in Python.
- The gate records a property test that fails on base as `red:property` and stores the
  runner's minimal counterexample in the ledger and the PR comment.

## Capabilities

### New Capabilities

### Modified Capabilities

- `authoring`: the `FOR ALL` step.
- `trace`: property test detection.
- `gate`: `red:property` evidence and counterexamples.

## Impact

Spec parser and validator, per-language test scanners in trace, gate evidence, ledger
schema (new optional `counterexample` field) and the PR comment.
