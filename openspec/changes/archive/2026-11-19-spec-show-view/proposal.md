## Why

To read a spec today, a person or an agent opens the Markdown file, and to see what proves one scenario they run `sj trace`, `sj ledger` and grep. OpenSpec has `openspec show <item>` to print one spec or change, and `openspec view` for an overview of the whole project. specjutsu replaces OpenSpec, so it needs both, and it can do better than OpenSpec here: it knows which tests cover each scenario and what their last runs showed.

## What Changes

- `sj spec show <item>` prints one capability spec, one change (proposal, deltas, task progress) or one scenario (its requirement, its WHEN/THEN, the tests mapped to it and its latest ledger evidence). `--json` prints the same data. An unknown item exits 1 and suggests the nearest names.
- `sj view` prints a one-screen project dashboard: capabilities and scenario counts, open changes with task progress, scenario evidence totals (proven, green-unproven, red, untested), orphan tests, and the number of archived changes. `--json` prints the same data. It is not interactive.

## Capabilities

### New Capabilities
None.

### Modified Capabilities
- `authoring`: adds `sj spec show`.
- `loop`: adds `sj view`.

## Impact

- `sj-cli`: new `spec show` and `view` subcommands.
- `sj-core`: reuses the spec parser, `trace` and the ledger reader. No new dependencies.
- README and manual command reference.

## Clarifications

- Should `sj spec show` also resolve archived change names? → Yes. After open changes and before capabilities, by folder name or by bare name when only one archived change has it.
