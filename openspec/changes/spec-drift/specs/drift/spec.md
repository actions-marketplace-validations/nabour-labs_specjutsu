## Purpose

Find where the spec and the code stop agreeing: code that a scenario's tests exercise changed while the scenario's spec and tests did not, and spec text that changed while its tests did not. It is deterministic, built from coverage and the diff.

## ADDED Requirements

### Requirement: A suite can declare a coverage command
A suite in `specjutsu.toml` MAY set `coverage`, a command that runs the suite's selected tests and writes an LCOV file. The command SHALL contain the placeholder `{lcov}`, which specjutsu replaces with the output path, and MAY contain `{tests}`, replaced with the selected tests in the runner's filter syntax. A `coverage` value without `{lcov}` SHALL be a config error naming the suite.

#### Scenario: DRIFT-001 A coverage command without {lcov} is rejected
- **WHEN** a suite sets `coverage = "cargo llvm-cov --lcov"`
- **THEN** `sj doctor` reports `suite cargo (.): coverage must contain {lcov}` as a config problem and exits 1

### Requirement: sj coverage map records the lines each scenario runs
`sj coverage map` SHALL run, for every scenario with mapped tests in a suite with `coverage`, that scenario's tests under the coverage command. It SHALL write `.specjutsu/coverage-map.json` holding the commit it was built on and, for each scenario ID, the source files and line ranges with a hit count above zero. Only files matching the package's `sources` SHALL be kept. Lines run by more than `[drift] shared_threshold` of the mapped scenarios (default 0.5) SHALL be dropped from every scenario and counted as shared.

#### Scenario: DRIFT-002 The map holds each scenario's lines
- **WHEN** AUTH-003's tests run lines 10–30 of `src/auth.rs` and AUTH-004's run lines 40–55, and `sj coverage map` runs
- **THEN** the map records `src/auth.rs` 10–30 for AUTH-003 and 40–55 for AUTH-004, and the commit at HEAD

#### Scenario: DRIFT-003 Lines most scenarios run are shared, not owned
- **WHEN** 10 scenarios are mapped and 8 of them run lines 1–9 of `src/lib.rs`
- **THEN** no scenario in the map owns `src/lib.rs` lines 1–9, and the output says `9 shared lines dropped`

#### Scenario: DRIFT-004 An up-to-date map is not rebuilt
- **WHEN** `sj coverage map` runs twice with no change to the working tree
- **THEN** the second run runs no test and prints `coverage map up to date`

### Requirement: Code drift is reported when owned lines change alone
`sj drift --base <ref>` SHALL load the map. For every line changed between base and head (modified or deleted on base), it SHALL find the scenarios that own that line. A scenario SHALL be reported as `code-drift <ID> <file>:<line>` (the first changed line) when its spec text is unchanged and none of its mapped test files changed in the range.

#### Scenario: DRIFT-005 Changing a scenario's code without its spec or tests is drift
- **WHEN** the map gives AUTH-003 `src/auth.rs` 10–30, the change edits line 15 of `src/auth.rs`, and neither AUTH-003's scenario text nor `tests/auth.rs` changed
- **THEN** `sj drift` reports `code-drift AUTH-003 src/auth.rs:15 — code its tests run changed; spec and tests did not`

#### Scenario: DRIFT-006 Updating the scenario's test clears the drift
- **WHEN** the same change also edits `tests/auth.rs`, which holds AUTH-003's test
- **THEN** `sj drift` reports nothing for AUTH-003

### Requirement: A changed scenario with unchanged tests is a stale test
`sj drift` SHALL report `stale-test <ID>` for every scenario classified as modified (GATE-001) none of whose mapped test files changed in the range. This check SHALL NOT need a coverage map.

#### Scenario: DRIFT-007 Editing a scenario's THEN without touching its test
- **WHEN** the change rewords AUTH-002's THEN line from `401` to `403` and no file holding an AUTH-002 test changed
- **THEN** `sj drift` reports `stale-test AUTH-002 — spec changed; its tests did not`

### Requirement: Drift works without a map, saying what it skipped
When `.specjutsu/coverage-map.json` is missing, `sj drift` SHALL run the stale-test check, print `no coverage map: code drift not checked (run sj coverage map)` and exit as if no code drift was found.

#### Scenario: DRIFT-008 No map means only stale-test runs
- **WHEN** there is no coverage map and the change modifies AUTH-002's spec text only
- **THEN** `sj drift` reports `stale-test AUTH-002` and the no-map line

### Requirement: A map from another commit is translated, not trusted blindly
When the map's commit differs from the base, `sj drift` SHALL translate the map's line ranges to base through `git diff <map-commit> <base>`, drop the lines changed in between, and print `coverage map from <short-sha>, <n> commits behind base`. When the map's commit is not an ancestor of base or is unknown, it SHALL behave as if there were no map and say why.

#### Scenario: DRIFT-009 Lines shift with insertions before them
- **WHEN** the map was built on a commit where AUTH-003 owns `src/auth.rs` 10–30, base inserts 5 lines at line 3 of `src/auth.rs`, and the change edits base line 20
- **THEN** `sj drift` reports `code-drift AUTH-003 src/auth.rs:20`

### Requirement: The gate reports drift, failing only when configured
`sj gate` SHALL include drift findings as warnings. With `[drift] fail = true`, each `code-drift` and `stale-test` SHALL fail the gate. In a change exempted as `refactor`, `code-drift` SHALL be printed as `info` and SHALL NOT fail the gate.

#### Scenario: DRIFT-010 Drift fails the gate only when configured
- **WHEN** the change has one `code-drift` for AUTH-003 and every changed scenario is proven, once with default config and once with `[drift] fail = true`
- **THEN** the first gate passes with `warn code-drift AUTH-003 …`, and the second fails with the reason `code-drift AUTH-003 src/auth.rs:15`

#### Scenario: DRIFT-011 A refactor exemption turns code drift into information
- **WHEN** `[drift] fail = true`, the change is `Exempt: refactor — extract pricing module`, and it has a `code-drift` for AUTH-003
- **THEN** the gate prints `info code-drift AUTH-003 …` and does not fail because of it

### Requirement: Drift output is compact and machine-readable
`sj drift` SHALL print `drift: none` when there is nothing to report, otherwise a summary line and one line per finding. `--json` SHALL print the same data with each finding's kind, scenario ID, file, line and the map's commit. It SHALL exit 0 when there are no findings or only warnings, 1 when `[drift] fail = true` and there are findings, and 2 on an environment error.

#### Scenario: DRIFT-012 JSON lists each finding
- **WHEN** `sj drift --base origin/main --json` runs on the change from DRIFT-005
- **THEN** stdout is one JSON object whose `findings` array has one entry with `kind` `code-drift`, `id` `AUTH-003`, `file` `src/auth.rs`, `line` 15 and the map's `commit`
