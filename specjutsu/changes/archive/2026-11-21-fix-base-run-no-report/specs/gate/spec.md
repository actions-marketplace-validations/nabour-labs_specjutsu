## ADDED Requirements

### Requirement: A base suite that writes no report is an environment error
When a non-Cargo suite's base run writes no parsable report, the gate SHALL treat it as an
environment error, not red: it SHALL name the package and runner, print the runner's last 20
output lines, suggest declaring `setup` or `red_on_base = "skip"`, and exit 2.

#### Scenario: GATE-046 A base run without installed dependencies exits 2
- **WHEN** the vitest suite of `apps/api` declares no `setup`, AUTH-004's test passes on base's
  `auth.ts`, and the base run fails with `Cannot find module 'vitest'` and writes no report
- **THEN** the gate prints `apps/api:vitest`, `Cannot find module 'vitest'` and a hint to declare `setup` on stderr
- **AND** exits 2 instead of recording AUTH-004 `red:compile`

#### Scenario: GATE-047 [keep] A base report with a failed test file is still red:compile
- **WHEN** the vitest suite of `apps/api` declares no `setup`, its base run writes a report whose only entry is a failed `src/auth.test.ts` file, and AUTH-004's test passes at head
- **THEN** AUTH-004 is recorded `red:compile` and the gate passes
