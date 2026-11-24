## ADDED Requirements

### Requirement: Only added and modified scenarios need a red run
`sj verify` and `sj task done` SHALL judge each scenario against a merge-base, classifying it
the same way the gate does. For `sj verify` that is the merge-base with `--base` when given,
otherwise with the default branch. `sj task done` always uses the default branch.

They SHALL require a red run only for **added** and **modified** scenarios. The following need
only a green latest run, and SHALL NOT be reported as unproven when they have one:

- an **unchanged** scenario, such as one copied without edits into a MODIFIED requirement;
- an **implemented** scenario (GATE-023). Its behaviour may predate the change, and the gate
  checks it on base.

The tests of these scenarios SHALL still run in `sj verify`, and a failing one SHALL still be
reported.

Every scenario in the change SHALL need a red run, as before, when there is no merge-base or
when the merge-base is HEAD itself (HEAD is already on the default branch). The same applies
when `sj task done` can't read the classification, and it then prints a warning. In `sj verify`,
an error while classifying is an environment error, as it is for the gate rules verify runs.

#### Scenario: LOOP-034 Verify and task done accept a linked existing scenario
- **WHEN** mode is `strict`, the merge-base has scenario STAFF-063 with no test, and task 4.1 of the change names STAFF-063 and adds its first test, which has only `passed` records in the ledger
- **THEN** `sj verify` prints no unproven line for STAFF-063, and `sj task done <change> 4.1` checks the box and exits 0

#### Scenario: LOOP-035 An unchanged scenario in a MODIFIED requirement needs no red run
- **WHEN** a change's `## MODIFIED Requirements` copies AUTH-002 with the same text as on the merge-base, where AUTH-002 already has a passing test, and the ledger has only `passed` records for it
- **THEN** `sj verify` prints no unproven line for AUTH-002, and still reports `failing … AUTH-002` if its test fails

#### Scenario: LOOP-036 An added scenario committed on the default branch still needs a red run
- **WHEN** mode is `strict`, a change that adds MATH-001 is committed on `main` itself, and MATH-001's test has only `passed` records in the ledger
- **THEN** `sj verify` prints `unproven MATH-001 (no red run recorded)`, and `sj task done` refuses the task naming MATH-001 and exits 1
