# Tasks

Each task group lands as two commits: the failing tests first (`test(IDs): red`), then the code (`feat(IDs): green`). Tests go under each crate's `tests/` directory so the gate can overlay them on base.

## 1. File classification (sj-core)

- [ ] 1.1 Replace `is_impl_file` with the fail-closed non-source list from design decision 1 (GATE-013, GATE-014, GATE-027, GATE-028). Verify: `evaluate` tests in `crates/sj-core/tests/gate.rs` for `web/app.ts` + `scripts/build.sh` (fails, both listed, sorted) and for `Cargo.toml` + `Cargo.lock` + `.github/workflows/ci.yml` (passes). Existing GATE-013 and GATE-014 tests stay green.

## 2. Unidentified scenarios (sj-core)

- [ ] 2.1 Fingerprint scenarios without IDs and match head against base by text as a multiset; fail each unmatched one as `unidentified <name> (<file>:<line>)` and add one `N scenarios without IDs` warning for the matched ones (GATE-024, GATE-025). Verify: fixture-repo tests in `crates/sj-core/tests/gate.rs` covering a new unidentified scenario, three unchanged ones (one warning line), a reworded THEN line, and an unidentified scenario moved from a change delta into `specs/` (still unchanged, GATE-002).

## 3. Runner check and exempted suites (sj-core + sj-cli)

- [ ] 3.1 Return an error from `gate::run` when the root has no `Cargo.toml`, before any git call (GATE-026). Verify: `crates/sj-cli/tests/gate.rs` runs `sj gate --base main` in a git repo with only `package.json` and asserts exit 2 and `cargo` in stderr. `sj gate --since` in the same repo still exits 0.
- [ ] 3.2 Run the full head suite and fail on failing tests whenever an exemption is declared and implementation files changed (GATE-029). Verify: a fixture with `Exempt: chore — …`, an edit to `src/lib.rs` and a failing test fails with `failing <path>`. The GATE-015 refactor test stays green.

## 4. Docs and self-hosting

- [ ] 4.1 Update the README "How the gate works" section (unidentified scenarios, non-source list, supported runners) and add a CHANGELOG `Unreleased` entry marking the classification change as breaking. Verify: built `sj gate --base origin/main` and `sj gate --since <root commit>` pass on this repo, and the false-pass repro from `docs/migration-readiness.md` now fails with exit 2.
