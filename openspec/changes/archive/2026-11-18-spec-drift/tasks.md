# Tasks

Each task group lands as two commits: the failing tests first (`test(IDs): red`), then the code (`feat(IDs): green`). Tests use a fixture runner whose coverage command copies canned LCOV files, so no real coverage tool is needed in CI.

## 1. The map

- [x] 1.1 `coverage` suite key with `{lcov}` and `{tests}` placeholders, checked by `sj doctor` (DRIFT-001).
- [x] 1.2 LCOV reader and `sj coverage map`: per-scenario runs, source filtering, shared lines, up-to-date detection (DRIFT-002..004). Verify: `crates/sj-cli/tests/coverage_map.rs`.

## 2. Drift checks

- [x] 2.1 `stale-test` and the no-map path (DRIFT-007, DRIFT-008).
- [x] 2.2 `code-drift` with the map, and the test-changed rule (DRIFT-005, DRIFT-006).
- [x] 2.3 Line translation through `git diff` for an older map (DRIFT-009).
- [x] 2.4 Output and JSON (DRIFT-012).

## 3. Gate

- [x] 3.1 Drift lines in `sj gate`, `[drift] fail`, refactor exemption as info (DRIFT-010, DRIFT-011).

## 4. Docs

- [x] 4.1 README "Spec drift" section with coverage commands for cargo, vitest, Jest and pytest, and the CI recipe that caches the map from the default branch; add the `DRIFT` prefix to `openspec/principles.md`; CHANGELOG entry (no-scenario: docs).
