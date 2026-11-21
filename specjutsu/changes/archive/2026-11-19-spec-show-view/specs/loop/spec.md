## ADDED Requirements

### Requirement: sj view prints a project dashboard
`sj view` SHALL print, in at most 40 lines for a project with up to 20 capabilities and 10 open changes: one line per capability with its scenario count; one line per open change with checked and total tasks and its scenarios counted as proven, green-unproven, red and untested (the same counts `sj status` uses); project totals for those four counts across all scenarios; the number of orphan tests (tests naming an ID that does not exist); and the number of archived changes. Beyond those sizes it SHALL show the first entries of each list and a `… +N more` line. `--json` SHALL print the same data. It SHALL read files and the ledger only, run no tests and wait for no input.

#### Scenario: LOOP-008 Dashboard shows changes and totals
- **WHEN** the project has capabilities `auth` (6 scenarios) and `billing` (4), open change `add-oauth-login` at 3/7 tasks with 2 proven and 1 untested scenario, 1 orphan test and 5 archived changes
- **THEN** `sj view` prints `auth  6 scenarios`, `billing  4 scenarios`, a line with `add-oauth-login`, `3/7 tasks`, `2 proven` and `1 untested`, `1 orphan test` and `5 archived`, and exits 0

#### Scenario: LOOP-009 Long lists are cut to fit
- **WHEN** the project has 30 capabilities
- **THEN** `sj view` prints 20 capability lines followed by `… +10 more` and at most 40 lines in all

#### Scenario: LOOP-010 JSON dashboard
- **WHEN** `sj view --json` runs on the project in LOOP-008
- **THEN** stdout is one JSON object with `capabilities`, `changes`, `totals`, `orphans` (1) and `archived` (5)

#### Scenario: LOOP-011 View never runs tests
- **WHEN** `sj view` runs with stdin closed and a runner executable that would fail if started
- **THEN** it exits 0 and the ledger file is unchanged
