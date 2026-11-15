## Why

`sj test` can only run `cargo test`. Your projects need vitest, pytest and Playwright, run per package, from a pnpm/Turbo or uv workspace.

## What Changes

- A runner adapter interface. A runner turns a suite plus a test selection into a command, and reads results from a **JUnit XML** report. Adapters: `vitest`, `jest`, `pytest`, `playwright`, plus a generic `junit` runner for any command that writes JUnit XML. `cargo` stays as it is.
- Each suite may declare a `command` override and `env`. Defaults come from the detected package manager or environment.
- `sj test [SCOPE]` runs every suite in scope. Scope can be a scenario ID, a package path or a suite. Output is failures-only across runners, in the same format as today.

## Capabilities

### New Capabilities
None.

### Modified Capabilities
- `run`: suites, runner adapters, JUnit results and multi-suite scope.

## Impact

- `sj-runners`: `Runner` trait, JUnit XML parser (`quick-xml`), five adapters.
- The gate keeps using Cargo for red-on-base until `polyglot-gate`.
