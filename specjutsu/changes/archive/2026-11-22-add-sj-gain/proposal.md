## Why

specjutsu's main claim is token economics: deterministic commands hand the agent a few lines instead
of whole folders and raw test logs. The roadmap promises a public token benchmark for v0.6, and the
public launch waits on it. Nothing measures the claim today. The design doc's `sj gain` was never
built, so neither users nor a benchmark harness can see how many bytes `sj` saved.

`sj gain` gives that number locally, using the same definition the benchmark repo will publish. Each
agent-loop command records the bytes it printed and the bytes of the raw material it stands in for.
`sj gain` totals them. specjutsu still never calls a model: tokens are estimated as bytes / 4, the
same rule the loop specs already use for their budgets.

## What Changes

- `sj test`, `sj verify`, `sj next` and `sj status` append one gain record per run to
  `.specjutsu/ledger/gain.jsonl`, holding the bytes printed and the raw-equivalent bytes.
- New command `sj gain`: totals per command (runs, printed bytes, raw bytes, % saved) and an
  estimated count of tokens saved.
- `sj gain --json` for machine-readable totals (used by the external benchmark harness).
- `sj gain --since <timestamp>` to total one session, and `sj gain --reset` to clear the records.
- README and manual document `sj gain`. The site keeps "measured" claims off until the public
  benchmark publishes numbers.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `loop`: gain records from the agent-loop commands, and the `sj gain` report.

## Impact

- `crates/sj-core`: a new gain module (record, read, total), hooked into the test, verify, next
  and status paths.
- `crates/sj-cli`: the `gain` subcommand and its flags.
- No new dependencies. The records live inside `.specjutsu/ledger/`, which `sj init` already
  git-ignores.
- Docs: README command reference and roadmap line, `site/manual.html`, CHANGELOG.
