# run Specification

## Purpose
Run a project's tests through `sj` with output small enough for an agent to read: only what failed, tied to the scenario IDs those tests prove.

## Requirements

### Requirement: sj test runs the project's tests
`sj test` SHALL run the project's Cargo tests from the project root, defaulting to the current directory or the directory given by `--root`. With no scope, it SHALL run every test in the workspace.

#### Scenario: RUN-001 Run the whole workspace with no scope
- **WHEN** `sj test` is run in a Cargo workspace with no scope argument
- **THEN** every test in the workspace runs
- **AND** the summary counts every test that ran

### Requirement: A scope narrows the run
`sj test SCOPE` SHALL treat a scope shaped like a scenario ID (`PREFIX-NNN`) as a scenario and run only the tests mapped to it, using the same mapping rules as `sj trace`. Any other scope SHALL be treated as a Cargo package name.

#### Scenario: RUN-002 A scenario ID runs only its mapped tests
- **WHEN** `sj test GATE-004` is run and two tests map to GATE-004
- **THEN** only those two tests run
- **AND** tests mapped to other scenarios do not run

#### Scenario: RUN-003 A scenario ID with no mapped test is an error
- **WHEN** `sj test` is given a well-formed scenario ID that no test maps to
- **THEN** no tests run
- **AND** sj reports that the scenario has no mapped test and exits 2

#### Scenario: RUN-004 Any other scope selects a package
- **WHEN** `sj test sj-core` is run in a workspace containing package `sj-core`
- **THEN** only the tests of package `sj-core` run

### Requirement: Output shows failures only
By default `sj test` SHALL print one summary line, then for each failing test only: its name, the scenario IDs it maps to, and a short excerpt of its failure output. Passing and ignored tests SHALL be counted in the summary but not listed.

#### Scenario: RUN-005 All passing prints a single line
- **WHEN** every test that ran passes
- **THEN** the output is one summary line with the passed and ignored counts and no per-test lines

#### Scenario: RUN-006 A failing test is listed with its IDs and an excerpt
- **WHEN** test `gate_004_untested` (mapped to GATE-004) fails with a panic message
- **THEN** the output lists `gate_004_untested` with `GATE-004`
- **AND** shows an excerpt of its failure output that includes the panic message
- **AND** does not list any passing test

#### Scenario: RUN-007 Long failure output is truncated
- **WHEN** a failing test produces more failure output than the excerpt limit
- **THEN** the excerpt is cut at the limit and marked as truncated

### Requirement: Build failures are reported as build failures
When the tests fail to compile, `sj test` SHALL report a build failure, show the compiler's error lines rather than an empty test summary, and SHALL NOT report the run as zero tests passing.

#### Scenario: RUN-008 A compile error is shown, not hidden
- **WHEN** a test file in scope does not compile
- **THEN** the summary says the build failed
- **AND** the output includes the compiler's `error` lines
- **AND** sj exits 1

### Requirement: Exit codes are stable
`sj test` SHALL exit 0 when every test that ran passed, 1 when any test failed or the build failed, and 2 when sj itself could not run (bad arguments, no Cargo project, unknown scenario).

#### Scenario: RUN-009 Exit codes distinguish pass, fail and error
- **WHEN** `sj test` is run against a passing suite, a suite with one failing test, and a directory that is not a Cargo project
- **THEN** it exits 0, 1 and 2 respectively

### Requirement: JSON output for tools
`sj test --json` SHALL print one JSON object holding the run's outcome, the counts, the build-failure flag, and for each test that ran: its path, result and mapped scenario IDs.

#### Scenario: RUN-010 --json lists every test with its IDs and result
- **WHEN** `sj test --json` runs a suite with one passing and one failing mapped test
- **THEN** stdout is a single JSON object
- **AND** it contains both tests with their results and scenario IDs
- **AND** the exit code is the same as without `--json`

### Requirement: Runners report results through JUnit XML
For the `vitest`, `jest`, `pytest`, `playwright` and `junit` runners, the system SHALL run the suite with a JUnit XML reporter writing to a temporary file. It SHALL read each test's result (`passed`, `failed`, `skipped`), its failure message and its location from that report. A traced test SHALL match a report entry by file and test name.

#### Scenario: RUN-011 A vitest failure is reported with its scenario and message
- **WHEN** `sj test` runs a vitest suite where `it("AUTH-003 rejects expired token")` fails with `expected 401, got 200`
- **THEN** the output lists `rejects expired token` with AUTH-003 and an excerpt containing `expected 401, got 200`, and exits 1

#### Scenario: RUN-012 A pytest pass prints one line
- **WHEN** `sj test python/worker` runs a pytest suite where all 12 tests pass
- **THEN** the output is one summary line counting 12 passed, and the exit code is 0

#### Scenario: RUN-013 No report means a build failure
- **WHEN** a vitest suite fails before writing its report (e.g. a TypeScript syntax error in a test file)
- **THEN** the run is reported as a build failure with the runner's error lines, not as zero tests, and exits 1

#### Scenario: RUN-014 The generic junit runner runs any command
- **WHEN** a suite declares `runner = "junit"`, `command = "make test"` and `report = "build/junit.xml"`
- **THEN** `sj test` runs `make test` in the package directory and reads results from `build/junit.xml`

### Requirement: Suites run with their package's tools
Each suite SHALL run in its package directory with a default command derived from the runner and the detected package manager or environment: `pnpm exec`, `npx` or `yarn` for Node runners, and `uv run` when the package has `uv.lock`, otherwise `python -m`, for pytest. A suite MAY override the command and add environment variables.

#### Scenario: RUN-015 pytest in a uv project runs through uv
- **WHEN** package `backend` has `uv.lock` and a pytest suite with no command override
- **THEN** the suite runs `uv run pytest` with the JUnit reporter in `backend/`

### Requirement: Scope selects scenarios, packages or suites across runners
`sj test SCOPE` SHALL accept a scenario ID (run only its linked tests, in every suite that has one), a package path (run every suite in it) or `<package>:<runner>` (run that suite). With no scope it SHALL run every suite. The summary SHALL count results across suites and name each suite that failed.

#### Scenario: RUN-016 A scenario with tests in two runners runs both
- **WHEN** CHAT-002 links to one vitest unit test in `apps/app` and one Playwright test in `apps/app/e2e`
- **THEN** `sj test CHAT-002` runs only those two tests, one per runner, and reports both

#### Scenario: RUN-017 A single suite can be selected
- **WHEN** `sj test apps/app:playwright` runs
- **THEN** only the Playwright suite of `apps/app` runs
