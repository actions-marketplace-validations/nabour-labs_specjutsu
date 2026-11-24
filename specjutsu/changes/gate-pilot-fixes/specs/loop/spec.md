## ADDED Requirements

### Requirement: An implemented scenario needs no red run
`sj verify` and `sj task done` SHALL NOT report an **implemented** scenario (GATE-023, judged
against the merge-base with the default branch) as unproven when its latest run is green. Its
behaviour may predate the change, and the gate checks it on base. Its tests SHALL still run in
`sj verify`, and a failing one SHALL still be reported. Added and modified scenarios still need
a red run.

#### Scenario: LOOP-034 Verify and task done accept a linked existing scenario
- **WHEN** mode is `strict`, the merge-base has scenario STAFF-063 with no test, and task 4.1 of the change names STAFF-063 and adds its first test, which has only `passed` records in the ledger
- **THEN** `sj verify` prints no unproven line for STAFF-063, and `sj task done <change> 4.1` checks the box and exits 0
