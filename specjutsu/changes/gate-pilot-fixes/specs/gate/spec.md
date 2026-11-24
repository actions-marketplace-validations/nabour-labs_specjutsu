## ADDED Requirements

### Requirement: A test helper missing on base is not red
When a target test fails to load on base, the gate SHALL check every file the change added
or modified that lies in a helper folder (a path segment named `testing`, `test-utils`, `test-helpers`,
`testutils`, `fixtures`, `__fixtures__`, `mocks` or `__mocks__`) and that was not overlaid. If
the base failure message for that test names such a file, by its path from the repository, the
package or the test's folder, with or without its extension, the gate MUST fail with
`not-overlaid <ID> <test name> (<file>; add it to the suite's support globs)` instead of
recording `red:compile`. Paths in the message SHALL match with `\` or `/` as the separator.

#### Scenario: GATE-056 A helper the change added but did not overlay fails the gate
- **WHEN** the change adds `apps/api/src/testing/admin.ts` and `apps/api/src/users.test.ts`, whose test for STAFF-073 imports `./testing/admin`, no `support` glob matches `admin.ts`, and the base run reports `users.test.ts` as a failed file with the message `Failed to load url ./testing/admin`
- **THEN** the gate fails with reason `not-overlaid STAFF-073 <test name> (apps/api/src/testing/admin.ts; add it to the suite's support globs)` and does not record STAFF-073 as `red:compile`

#### Scenario: GATE-057 [keep] An overlaid helper lets the test fail on an assertion
- **WHEN** the same change runs with `support = ["apps/api/src/testing/**"]` on the vitest suite, and STAFF-073's test loads on base and fails an assertion
- **THEN** STAFF-073 is recorded `red:assert`

#### Scenario: GATE-062 A helper the change modified but did not overlay fails the gate
- **WHEN** base already has `apps/api/src/testing/index.ts`, the change adds an export `makeAdmin` to it and a test for STAFF-074 in `apps/api/src/staff.test.ts` that imports it, no `support` glob matches the file, and the base run reports `staff.test.ts` as a failed file with the message `The requested module './testing/index.ts' does not provide an export named 'makeAdmin'`
- **THEN** the gate fails with reason `not-overlaid STAFF-074 <test name> (apps/api/src/testing/index.ts; add it to the suite's support globs)`

#### Scenario: GATE-058 [keep] A new source module missing on base is still red:compile
- **WHEN** the change adds `apps/api/src/usage.ts` and a test for USAGE-001 that imports it, and the base run fails to load the test because `./usage` does not exist
- **THEN** USAGE-001 is recorded `red:compile`

### Requirement: An implemented scenario that passes on base is linked
An **implemented** scenario (GATE-023) whose tests all pass on base and at head SHALL pass the
gate, recorded as `green:linked`: the behaviour existed before the change, and the change only
links a test to it. A `green:linked` scenario SHALL NOT count as specifying a source change.
`--json` SHALL list it in `proven` with `"red": "green:linked"`, and `--format markdown` SHALL
list it under a `Linked existing behaviour` heading, apart from the scenario table.

#### Scenario: GATE-059 A test linked to existing behaviour passes the gate
- **WHEN** base has scenario STAFF-063 with no test, and the change adds a test for STAFF-063 that passes on base and at head, and changes no source file
- **THEN** the gate passes and `--json` lists STAFF-063 in `proven` with `"red": "green:linked"`

#### Scenario: GATE-060 A linked scenario does not cover a source change
- **WHEN** the same change also edits `apps/api/src/users.ts`, STAFF-063 is its only implemented, added or modified scenario, and it declares no exemption
- **THEN** the gate fails with reason `unspecified change` listing `apps/api/src/users.ts`

## MODIFIED Requirements

### Requirement: Implementing an existing scenario puts it in scope
A scenario that is unchanged in text but had no linked test on base and has one
at head SHALL be classified as **implemented** and treated like an added
scenario by every gate rule, so a change that implements a spec merged earlier
must prove it. The one exception is a test that passes on base, which is
`green:linked` rather than `green-on-base`. When an implemented scenario has several tests and
at least one fails on base, that failure proves the scenario, and its tests that pass on base
are accepted.

#### Scenario: GATE-023 Implementing a previously untested scenario is gated
- **WHEN** base already has scenario MATH-001 with no test, and the change adds a test for MATH-001 and edits `src/lib.rs`
- **THEN** MATH-001 is classified as implemented, the change is not reported as an unspecified change, and MATH-001 must pass red-on-base

### Requirement: Each suite declares its red-on-base policy
A suite SHALL accept `red_on_base = "run"` (default), `red_on_base = "assert"`, or `red_on_base = "skip"` with a `reason`. In an `assert` suite, only an assertion failure on base SHALL count as red, and a target test that fails to compile or load on base SHALL fail the gate with `compile-red <ID> <test name> (suite requires an assertion failure on base)`. Target tests in a skipped suite SHALL be reported `unverifiable (<reason>)` and fail the gate unless it runs with `--allow-unverifiable`. Head runs are never skipped.

#### Scenario: GATE-034 An e2e suite that skips red-on-base is unverifiable
- **WHEN** the Playwright suite declares `red_on_base = "skip"`, `reason = "needs staging API"`, and CHAT-002's only test is in it
- **THEN** the gate fails with `unverifiable CHAT-002 sends a message (needs staging API)`
- **AND** with `--allow-unverifiable` it passes with that line as a warning, provided the test passes at head

#### Scenario: GATE-061 An assert suite rejects a compile red
- **WHEN** the vitest suite of `apps/api` declares `red_on_base = "assert"`, AUTH-004's test fails to load on base, and passes at head
- **THEN** the gate fails with reason `compile-red AUTH-004 <test name> (suite requires an assertion failure on base)`

### Requirement: Assigning an ID to an existing scenario is not a change
A scenario with an ID at head whose ID is absent on base SHALL be classified **unchanged**, not added, when its text without the ID matches a scenario without an ID on base. It then follows the unchanged-scenario rules, including **implemented** (GATE-023) when it gains its first test in the same change.

#### Scenario: GATE-036 An ID-only commit passes the gate
- **WHEN** base holds 300 scenarios without IDs and the change only runs `sj ids --write`, giving each an ID
- **THEN** all 300 are classified unchanged, they appear as warnings (gaps), and the gate passes

#### Scenario: GATE-037 Adding an ID and a test at once is still gated
- **WHEN** the change gives the existing scenario `Valid login` the ID AUTH-001, adds a test for AUTH-001 that passes on base, and edits `src/auth.rs`
- **THEN** AUTH-001 is classified implemented and recorded `green:linked`, and the gate fails with reason `unspecified change` listing `src/auth.rs`
