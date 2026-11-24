## ADDED Requirements

### Requirement: sj-propose covers specs drafted from existing tests
The `sj-propose` skill SHALL have a section on drafting specs from existing tests that tells the
agent to run `sj spec infer` on one area at a time, read each test at the file and line its
marker names, and replace both markers with a concrete WHEN and THEN before running
`sj spec check`.

#### Scenario: SKILL-021 sj-propose names the infer workflow
- **WHEN** the installed `sj-propose` skill is read
- **THEN** it has a `From existing tests` section that names `sj spec infer`, `--write`, replacing the `[NEEDS CLARIFICATION` markers from the test, and `sj spec check`
