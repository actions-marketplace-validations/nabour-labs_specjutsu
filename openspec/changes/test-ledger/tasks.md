# Tasks

Each task group lands as two commits: the failing tests first (`test(IDs): red`), then the code (`feat(IDs): green`). Tests go under each crate's `tests/` directory so the gate can overlay them on base.

## 1. Cargo runner (sj-runners)

- [x] 1.1 Build check with `cargo test --no-run`, collecting the `error` lines and the `-->` line after each (capped) when it fails (RUN-008). Verify: `crates/sj-runners/tests/` covers a fixture crate that does not compile, reports a build failure with an `error` line, and reports no results.
- [x] 1.2 Parse libtest `---- <path> stdout ----` sections into per-test failure text, keeping `Run` backward-compatible for the gate (RUN-006). Verify: the libtest sample tests in `crates/sj-runners/tests/libtest.rs` pass, and `cargo test -p sj-core --test gate` stays green.

## 2. Test runs (sj-core)

- [x] 2.1 Resolve the scope: a scenario ID maps to test names through the trace mapping, an ID with no tests is an error, anything else is a package (RUN-002, RUN-003, RUN-004). Verify: `crates/sj-core/tests/run.rs` tests on a fixture repo.
- [x] 2.2 Run in scope with the exact post-filter, and join each result to its scenario IDs into a runner-neutral `TestRun` (RUN-001, RUN-002). Verify: in a fixture where `foo_001_adds` and `xfoo_001_adds` both exist (the libtest filter matches both), `FOO-001` returns only `foo_001_adds`.
- [x] 2.3 Excerpt truncation at 20 lines with a truncated marker (RUN-007). Verify: a unit test on a 50-line failure.

## 3. Ledger (sj-core)

- [x] 3.1 Working-tree hash through a throwaway index (LEDGER-004, LEDGER-005). Verify: in a fixture repo, an edit changes the hash, and `git status --porcelain` is byte-identical before and after.
- [x] 3.2 `Record`, a single-write append to `.specjutsu/ledger/runs.jsonl`, and a load that skips and counts bad lines (LEDGER-001, LEDGER-002, LEDGER-012). Verify: `crates/sj-core/tests/ledger.rs`.
- [x] 3.3 Record build failures against the mapped tests for scenario scopes, and nothing for package or workspace scopes (LEDGER-003). Verify: a fixture with a compile error under `sj test`-equivalent core calls.
- [x] 3.4 Per-scenario report: latest result, and red-before-green proven on a different tree in file order (LEDGER-008, LEDGER-009, LEDGER-010). Verify: pure tests on hand-built record lists.

## 4. CLI (sj-cli)

- [ ] 4.1 `sj test [SCOPE] [--root --spec-root --json --no-ledger]`: failures-only text, JSON, exit codes 0/1/2, a warning and no recording outside git (RUN-005, RUN-006, RUN-009, RUN-010, LEDGER-006, LEDGER-007). Verify: `crates/sj-cli/tests/test_cmd.rs` with `assert_cmd` on fixture projects.
- [ ] 4.2 `sj ledger [ID] [--json]` compact report and JSON (LEDGER-008, LEDGER-011). Verify: `crates/sj-cli/tests/ledger_cmd.rs`.
- [ ] 4.3 Document `sj test` and `sj ledger` in README.md (commands and the ledger's location and format) and add a CHANGELOG `Unreleased` entry. Verify: the README examples run as written against this repo.

## 5. Self-hosting

- [ ] 5.1 Run `sj test` and `sj ledger` on this repo. Verify: `sj test` exits 0 with one summary line, `sj test GATE-004` runs only the GATE-004 tests, and `sj trace --strict` and `sj gate --base origin/main` pass with the new scenarios.
