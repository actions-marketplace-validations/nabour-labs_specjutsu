## Why

`sj spec archive` refuses a change whose MODIFIED requirement carries an unchanged scenario
as `[keep]`. Marking carried scenarios `[keep]` is how a change avoids needing red runs for
them (LOOP-020), so every such change hits this; more-adapters-3 had to strip the markers by
hand before archiving.

Kind: bug

## Current Behaviour

With `#### Scenario: ADAPT-011 Installing two agents writes AGENTS.md once` in the main spec
and `#### Scenario: ADAPT-011 [keep] Installing two agents writes AGENTS.md once` in the
MODIFIED block, archive fails with `MODIFIED failed for header "### Requirement: …" - current
spec contains scenario(s) not present in the modified block: "ADAPT-011 …"`. The check compares
heading text literally.

## Expected Behaviour

The check ignores the `[keep]` marker on either side, so the scenario matches and archive
succeeds. The merged spec has the MODIFIED block as written in the delta, `[keep]` included,
the same way ADDED `[keep]` scenarios already land in main specs (for example ADAPT-044).

## Unchanged Behaviour

A MODIFIED block that really drops a scenario is still refused, naming the dropped scenario.

## Impact

`crates/sj-core/src/authoring/archive.rs` (MODIFIED scenario check).
