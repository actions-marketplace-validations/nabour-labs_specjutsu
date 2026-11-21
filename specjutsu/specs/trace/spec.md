# trace Specification

## Purpose
Map every spec scenario to the tests that cover it, by stable scenario ID, and report scenarios that have no test (gaps) and tests that name an unknown ID (orphans).

## Requirements

### Requirement: Scenario IDs are parsed from spec headings
The system SHALL extract a scenario ID from every `#### Scenario:` heading whose
name begins with an ID of the form `PREFIX-NNN` (uppercase letters, a hyphen,
one or more digits), recording the spec file and line.

#### Scenario: TRACE-001 Parse scenario IDs from headings
- **WHEN** a spec file contains `#### Scenario: AUTH-003 Reject expired token`
- **THEN** the parser returns scenario `AUTH-003` named `Reject expired token` with its file path and line number

#### Scenario: TRACE-002 Headings without an ID are reported, not dropped
- **WHEN** a spec file contains `#### Scenario: Reject expired token` with no ID
- **THEN** the parser returns it as an unidentified scenario with its file and line

### Requirement: Spec files are discovered under the spec root
The system SHALL read `specs/**/spec.md` and `changes/*/specs/**/spec.md` under
the spec root and SHALL ignore `changes/archive/`.

#### Scenario: TRACE-003 Discover current and change specs, skip archive
- **WHEN** the spec root holds `specs/a/spec.md`, `changes/x/specs/b/spec.md` and `changes/archive/old/specs/c/spec.md`
- **THEN** scenarios from the first two files are returned and none from the archive

### Requirement: Tests are matched to scenario IDs
The system SHALL match a Rust test to a scenario when the test function name
contains the ID with `-` replaced by `_`, compared case-insensitively, or when a
`// spec: <ID>` comment appears within the lines directly above the test.

#### Scenario: TRACE-004 Match IDs in Rust test names and spec comments
- **WHEN** a test file contains `fn auth_003_rejects_expired()` and another test preceded by `// spec: AUTH-004`
- **THEN** the matcher returns links AUTH-003 → `auth_003_rejects_expired` and AUTH-004 → that test, each with file and line

### Requirement: The trace report lists gaps and orphans
The system SHALL report every scenario with its linked tests, every scenario with
no test, and every test linked to an ID that no scenario defines.

#### Scenario: TRACE-005 Report mapped, gaps and orphans
- **WHEN** scenarios AUTH-001 and AUTH-002 exist and tests reference AUTH-001 and AUTH-009
- **THEN** the report shows AUTH-001 as mapped, AUTH-002 as a gap and AUTH-009 as an orphan

#### Scenario: TRACE-006 sj trace prints a compact report and supports JSON
- **WHEN** `sj trace` runs in a repo with one gap
- **THEN** it prints a summary line with counts and one line per gap, exiting 0, and `--json` prints the same data as JSON
- **AND** with `--strict` it exits 1

### Requirement: Test files are found through each suite's globs
The system SHALL find test files for each configured suite by matching its `tests` globs under the package path, respecting `.gitignore`. When a suite declares no globs, the runner's defaults SHALL apply: vitest and Jest `**/*.{test,spec}.{ts,tsx,js,jsx,mts,cts}`, Playwright `**/*.spec.ts` under its configured test directory, pytest `**/test_*.py` and `**/*_test.py`, and cargo the current Rust rule.

#### Scenario: TRACE-007 Co-located and tests-folder files are both found
- **WHEN** package `apps/api` has a vitest suite with default globs and contains `src/auth.test.ts` and `tests/unit/models.test.ts`
- **THEN** both files are scanned and their tests belong to package `apps/api`, suite `vitest`

#### Scenario: TRACE-008 Files outside every suite are not scanned
- **WHEN** `scripts/seed.ts` contains a call `test("AUTH-003 x")` and matches no suite's globs
- **THEN** it produces no test and no orphan

### Requirement: TypeScript and JavaScript tests link by title, describe or comment
In vitest, Jest and Playwright files the system SHALL treat each `it(…)`, `test(…)` and their `.only`/`.skip`/`.each` forms with a string-literal title as a test. A test SHALL link to every scenario ID that appears in its own title, in the title of any enclosing `describe(…)`, or in a `// spec: <IDs>` comment on the lines directly above the test or its `describe`. A test's name is its titles joined with ` > `.

#### Scenario: TRACE-009 ID in an it() title
- **WHEN** `auth.test.ts` contains `it("AUTH-003 rejects expired token", …)`
- **THEN** the trace links AUTH-003 to the test `AUTH-003 rejects expired token` with its file and line

#### Scenario: TRACE-010 ID in a describe() title applies to every test inside
- **WHEN** `describe("AUTH-004 refresh", () => { it("issues a new token"); it("revokes the old one") })`
- **THEN** both tests link to AUTH-004, named `AUTH-004 refresh > issues a new token` and `AUTH-004 refresh > revokes the old one`

#### Scenario: TRACE-011 Spec comment above a Playwright test
- **WHEN** an e2e file has `// spec: CHAT-002, CHAT-003` on the line above `test("sends a message", …)`
- **THEN** that test links to CHAT-002 and CHAT-003

#### Scenario: TRACE-012 Skipped tests are found and marked
- **WHEN** a file contains `it.skip("AUTH-005 pending case", …)`
- **THEN** the trace links AUTH-005 to that test and marks it `skipped`

### Requirement: Python tests link by name, marker or comment
In pytest files the system SHALL treat functions named `test*` at module level or inside `Test*` classes as tests. A test SHALL link to a scenario ID that appears in its name (`-` written as `_`, any case), in a `@pytest.mark.spec("<ID>", …)` decorator, or in a `# spec: <IDs>` comment on the lines directly above the function or its class.

#### Scenario: TRACE-013 ID in a pytest function name
- **WHEN** `tests/test_inventory.py` defines `def test_inv_002_removes_item():`
- **THEN** the trace links INV-002 to `test_inv_002_removes_item`

#### Scenario: TRACE-014 pytest marker and class-level comment
- **WHEN** `# spec: INV-003` sits above `class TestDiff:`, whose method `test_added` also has `@pytest.mark.spec("INV-004")`
- **THEN** `TestDiff::test_added` links to INV-003 and INV-004

### Requirement: Trace reports which package and suite each test belongs to
Every linked test in the trace report SHALL carry its package path and suite runner. `sj trace` SHALL warn once per package that has files matching its `sources` globs but no suite.

#### Scenario: TRACE-015 Package and suite appear in JSON
- **WHEN** `sj trace --json` runs in the polyglot fixture
- **THEN** each test entry has `package` and `suite` fields, e.g. `apps/api` and `vitest`

#### Scenario: TRACE-016 A package without a suite warns
- **WHEN** package `packages/ui` has `src/button.tsx` and no suite
- **THEN** `sj trace` prints `warn    no test suite: packages/ui`
