## Why

The gate proves that the scenarios a change touches are tested. It says nothing about the scenarios a change *doesn't* touch. An agent asked to fix a bug in the cart edits `src/pricing.rs`, which also decides how discounts work. The discount scenarios and their tests are unchanged, the tests still pass (or were adjusted), and the spec now describes behaviour the code no longer has. That is spec drift. It is the failure that makes teams stop trusting their specs, and agents cause it faster than people do, because they edit wherever the fix is easiest.

Detecting drift by reading spec and code would need an LLM, which specjutsu never calls. But most drift can be found deterministically:

- **Which code belongs to which scenario?** Coverage answers it: the lines a scenario's tests run.
- **Did that code change while the scenario's spec and tests stayed the same?** The diff answers it.

The same map shows the other direction for free: a scenario whose spec text changed while none of its tests did.

## What Changes

- Each suite can declare a `coverage` command that writes an LCOV file (`cargo llvm-cov`, `vitest --coverage`, `pytest --cov`).
- `sj coverage map` runs each scenario's tests under coverage and writes `.specjutsu/coverage-map.json`: for each scenario, the source lines its tests run, recorded against a commit. Lines that most scenarios run (setup, `main`) are dropped as shared.
- `sj drift --base <ref>` reports:
  - `code-drift <ID> <file>:<line>`: lines a scenario's tests run changed, while its spec and tests did not.
  - `stale-test <ID>`: a scenario's spec text changed, while none of its tests did. This needs no map.
- `sj gate` reports drift as warnings by default; `[drift] fail = true` makes them failures. A `refactor` exemption turns code drift into information lines.

## Capabilities

### New Capabilities
- `drift`: the per-scenario coverage map and the drift checks.

### Modified Capabilities
None. The suite's new `coverage` key is specified in `drift`. The gate's existing rules are unchanged; it gains drift lines.

## Impact

- `sj-core`: an LCOV reader, the map, line translation through `git diff`, and the drift checks.
- `sj-runners`: run a suite's tests for one scenario under its coverage command.
- `sj-cli`: `sj coverage map`, `sj drift`; `sj gate` includes drift lines.
- `specjutsu.toml`: `coverage` on a suite, a `[drift]` table.
- The action (from `gate-action`): on pushes to the default branch it can build the map and save it with `actions/cache`, and PR runs restore it. That's documented in this change and automated later.
- `openspec/principles.md`: adds the `DRIFT` prefix.
