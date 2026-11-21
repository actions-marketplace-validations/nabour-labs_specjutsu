## ADDED Requirements

### Requirement: Keep scenarios must pass on base and at head
A scenario whose heading carries `[keep]` after its ID SHALL be treated as behaviour the
change must not alter. The gate MUST require a mapped test for it, MUST run that test on the
base code with the change's test files overlaid, and MUST record it as `green:keep` when it
passes on base and at head.

#### Scenario: GATE-039 A keep test that passes on base and head is accepted
- **WHEN** a change adds `#### Scenario: PAY-012 [keep] Refunds under $100 auto-approve` and its test passes on base and at head
- **THEN** the gate passes PAY-012 with evidence `green:keep`

#### Scenario: GATE-040 A keep test that fails on base fails the gate
- **WHEN** the test for keep scenario PAY-012 fails when run against the base code
- **THEN** the gate fails with reason `keep-red-on-base PAY-012 <test name>` and a hint that the scenario describes new behaviour, not kept behaviour

#### Scenario: GATE-041 A keep test that fails at head fails the gate
- **WHEN** the test for keep scenario PAY-012 passes on base and fails at head
- **THEN** the gate fails with reason `keep-broken PAY-012 <test name>`

#### Scenario: GATE-042 A keep scenario without a test fails the gate
- **WHEN** keep scenario PAY-012 has no mapped test
- **THEN** the gate fails with reason `untested PAY-012`, the same as any other scenario

#### Scenario: GATE-043 Keep evidence appears in the JSON output and PR comment
- **WHEN** `sj gate --json` runs on a change with one keep scenario and one normal scenario
- **THEN** the keep scenario's entry has `"kind": "keep"` and `"evidence": "green:keep"`, and the PR comment lists it under a `Kept behaviour` heading
