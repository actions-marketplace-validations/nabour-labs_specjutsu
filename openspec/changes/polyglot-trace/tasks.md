# Tasks

Each task group lands as two commits: the failing tests first (`test(IDs): red`), then the code (`feat(IDs): green`). Tests go under each crate's `tests/` directory so the gate can overlay them on base.

## 1. JS/TS scanner (sj-core)

- [x] 1.1 Extend `TestFn` with `name_ids`, `skipped`, `package` and `suite`, and switch `build_report` to `name_ids` (design decision 2). Add `testscan::scan_js` with the tokenizer and call stack from design decision 3 (TRACE-009, TRACE-010, TRACE-011, TRACE-012). Verify: `crates/sj-core/tests/scan_js.rs` covers the four scenarios plus `test.describe`, `.each`, `.only`, nested describes, a JSX file, a regex literal containing `it(`, a template literal with `${}` and a commented-out test. The existing TRACE-001..006 and GATE tests stay green.

## 2. Python scanner (sj-core)

- [x] 2.1 `testscan::scan_python` from design decision 4 (TRACE-013, TRACE-014). Verify: `crates/sj-core/tests/scan_python.rs` covers module-level and class methods, `async def`, a non-`Test` class (ignored), the marker with several IDs, a comment above the class, and `@pytest.mark.skip`.

## 3. Suite discovery (sj-core)

- [x] 3.1 `testscan::scan_project` with ownership-first suite selection, longest-literal-prefix tie-breaking and per-runner defaults (design decision 1). Change the cargo glob in `Config::default_for` and `adopt` to `**/*.rs` (TRACE-007, TRACE-008). Verify: `crates/sj-core/tests/scan_project.rs` builds a temp tree with `apps/api` (vitest, default globs) holding `src/auth.test.ts` and `tests/unit/models.test.ts`, an `apps/app` e2e file claimed by Playwright, a pytest package, and `scripts/seed.ts` outside every suite, and checks package and suite on each test and nothing from `scripts/`.

## 4. Report and CLI (sj-core + sj-cli)

- [x] 4.1 `TestRef` fields, `Report.warnings`, the `unsuited` packages, and `sj trace` on `scan_project` (design decision 5) (TRACE-015, TRACE-016). Verify: `crates/sj-cli/tests/trace_polyglot.rs` runs `sj trace --json` in the same tree, finds `package`/`suite` on entries (`apps/api`, `vitest`), and finds `warn    no test suite: packages/ui` in the compact output. `sj trace` on this repo still reports the same mapped count.

## 5. Docs and self-hosting

- [ ] 5.1 Update the README "How tracing works" section with the JS/TS and Python link forms, update this repo's `specjutsu.toml` cargo glob, and add a CHANGELOG entry. Verify: built `sj trace` and `sj gate --base origin/main` pass on this repo. `sj trace` on scratch copies of the three migration targets' adopted configs (read-only on the projects) reports tests found per suite, and the misses get noted in the PR.
