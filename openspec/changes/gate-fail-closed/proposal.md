## Why

`sj gate` passes changes it cannot check. On a TypeScript repo, a PR that edits `src/a.ts` and adds a scenario with no ID and no test gets `gate: pass — no changed scenarios to prove`, exit 0 (reproduced; see `docs/migration-readiness.md`, finding 1). There are three causes. Only `*.rs` files count as implementation. Scenarios without IDs are invisible to the gate. Exemptions other than `refactor` skip the test run entirely. A gate that passes what it can't check is worse than no gate, and it must be fixed before specjutsu is pointed at any non-Rust project.

## What Changes

- An added or modified scenario without an ID fails the gate (`unidentified <name> (<file>:<line>)`). Unchanged scenarios without IDs only produce one summary warning, so existing projects can adopt specjutsu gradually.
- The gate exits 2 when the repository has no test runner it supports (today: no `Cargo.toml` at the root), instead of passing.
- Which files count as implementation is inverted. Before, only `*.rs` counted. Now every changed file counts unless it is a test, a spec, documentation, CI/repo metadata or a manifest/lockfile. An unrecognised file type now needs a scenario or an exemption. **BREAKING** for changes that touch, for example, shell scripts or web assets without an exemption.
- An exempted change that touches implementation files runs the full test suite at head, for every exemption kind, not only `refactor`.

## Capabilities

### New Capabilities
None.

### Modified Capabilities
- `gate`: the unspecified-change rule uses a fail-closed definition of implementation files. New requirements cover unidentified scenarios, unsupported repositories, and running the suite for every exempted source change.

## Impact

- `crates/sj-core/src/gate.rs`: file classification, unidentified-scenario detection, runner check, head-run condition.
- `crates/sj-core/src/spec.rs`: fingerprints for scenarios without IDs.
- `crates/sj-core/tests/gate.rs`, `crates/sj-cli/tests/gate.rs`: new GATE-024…GATE-029 tests.
- This repo: future PRs that touch `.github/install-sj.sh` stay non-source (under `.github/`). PRs that touch other non-Rust files under `site/` or `scripts/` will need an exemption.
- No new dependencies.
