## ADDED Requirements

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
