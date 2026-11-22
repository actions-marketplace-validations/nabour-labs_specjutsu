## ADDED Requirements

### Requirement: A keep scenario needs no red run
`sj verify` and `sj task done` SHALL NOT report a `[keep]` scenario as unproven, since it must pass on base and so never has a red run. Its tests SHALL still run in `sj verify`, and a failing one SHALL still be reported.

#### Scenario: LOOP-020 Verify and task done accept a passing keep scenario
- **WHEN** a change adds AUTH-005 `[keep]` and AUTH-006, both with passing tests and no red run recorded
- **THEN** neither `sj verify` nor `sj task done` prints an unproven line for AUTH-005

#### Scenario: LOOP-021 [keep] A passing scenario without a red run is still unproven
- **WHEN** a change adds AUTH-006, not `[keep]`, with a passing test and no red run recorded
- **THEN** `sj verify` and `sj task done` print `unproven AUTH-006 (no red run recorded)`
