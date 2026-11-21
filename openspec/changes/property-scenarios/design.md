## Context

The gate proves that a test failed before the code and passes after. For a rule over all
inputs, an example test passing proves little. Property-based testing libraries exist for
every runner specjutsu supports, and each prints a shrunk counterexample on failure.

## Goals / Non-Goals

**Goals:**
- Writing a property in the spec obliges a property-based test, checked in CI.
- The counterexample from the red run is kept as evidence that the property had teeth.

**Non-Goals:**
- Generating property tests. The agent writes them; specjutsu never calls a model.
- Supporting every property library. The four named cover the supported runners; others
  can be added per runner later.

## Decisions

- **A `FOR ALL` step, not a heading marker.** The domain of the property is part of the
  behaviour and belongs in the steps, where reviewers read it. OpenSpec treats an unknown
  bold step as prose, so the format stays compatible.
- **Detection by token in the test body.** It is the same lightweight scanning trace uses
  for names and comments. Rejected: running the test to find out, which makes trace slow.
- **Counterexample from stdout, first line only.** It is enough to show the property was
  violated, and keeps the ledger small.

## Risks / Trade-offs

- [A property test with a trivially narrow generator] → out of scope; review and `sj audit`
  assertion checks still apply.
