# Tasks

Each task group lands as two commits: the failing tests first (`test(IDs): red`), then the code (`feat(IDs): green`). Tests go under each crate's `tests/` directory so the gate can overlay them on base.

## 1. Suite keys and invocations (sj-core)

- [x] 1.1 `command`, `report` and `env` on `Suite`, the `junit` runner, their validation and `to_toml` (design decision 1). Build the per-runner `Invocation` from design decision 2 (RUN-014, RUN-015). Verify: `crates/sj-core/tests/invocation.rs` checks the argv, cwd and env for vitest under pnpm, yarn and npm lockfiles; Jest's env; Playwright's `file:line`; pytest with and without `uv.lock` (in the package and in a parent); a `command` override on vitest; and `junit` through the shell. Config tests reject `junit` without `command`/`report` and `report` on vitest, and round-trip the new keys.

## 2. JUnit reports (sj-runners + sj-core)

- [x] 2.1 `sj_runners::junit::parse` with `quick-xml`, and `match_cases` from design decision 3 (RUN-011, RUN-013). Verify: `crates/sj-runners/tests/junit.rs` parses the captured vitest, pytest (xunit1 and xunit2) and Playwright samples in `tests/data/` (failure, skipped, CDATA, the file-level vitest error). `crates/sj-core/tests/match_cases.rs` joins them to traced tests, covering `.each` placeholders, pytest parameters and classes, a path-suffix match, and a file-level error turning selected tests into `build-failed`.

## 3. sj test across suites (sj-core + sj-cli)

- [x] 3.1 Scope parsing, the run loop, `SuiteRun`, per-suite build failures, `records_for` and the output from design decisions 4–6 (RUN-011, RUN-012, RUN-013, RUN-014, RUN-016, RUN-017). Verify: `crates/sj-cli/tests/test_polyglot.rs` uses fake `pnpm` and `python3` executables (Unix) that log their argv and write canned reports. It covers the RUN-011 failure output with AUTH-003 and `expected 401, got 200`, the RUN-012 one-line pass for 12 tests, RUN-013's missing report as a build failure, RUN-016 running one vitest and one Playwright test for CHAT-002 with the right selection arguments, and RUN-017 running only `apps/app:playwright`. A `junit` suite whose command copies a report runs on every OS (RUN-014). The existing RUN-001..010 and LEDGER tests stay green.

## 4. Docs and self-hosting

- [ ] 4.1 README "Test runs" section on suites, scopes, launchers and the `junit` runner, the CHANGELOG entry, and the roadmap row split in `docs/migration-readiness.md`. Verify: built `sj test` and `sj gate --base origin/main` pass on this repo. `sj test` on a scratch package that symlinks `chrome-agent`'s `node_modules` runs a real vitest suite and a real Playwright suite. `sj test` on a scratch pytest package runs a real pytest suite through FridgeSense's backend `.venv`. Both show the right failures and IDs.
