## ADDED Requirements

### Requirement: Scenario headings may carry a keep marker
The spec parser SHALL accept `[keep]` between a scenario's ID and its name, and MUST store
it on the scenario as `keep`. Any other bracketed word in that position SHALL be a validation
error naming the file and line.

#### Scenario: SPEC-038 A keep marker is parsed
- **WHEN** a delta spec contains `#### Scenario: PAY-012 [keep] Refunds under $100 auto-approve`
- **THEN** `sj spec show PAY-012 --json` reports `"keep": true` and the name `Refunds under $100 auto-approve`

#### Scenario: SPEC-039 An unknown marker is rejected
- **WHEN** a delta spec contains `#### Scenario: PAY-013 [maybe] Partial refunds`
- **THEN** `sj spec validate` exits 1 with `unknown scenario marker [maybe]` and the file and line

### Requirement: sj spec new can scaffold a bug fix
`sj spec new <name> --bug` SHALL write a proposal.md with `## Current Behaviour`,
`## Expected Behaviour` and `## Unchanged Behaviour` sections and a `Kind: bug` line. A change
is a bug change when its proposal.md has that line.

#### Scenario: SPEC-040 A bugfix change is scaffolded
- **WHEN** the user runs `sj spec new fix-refund-rounding --bug`
- **THEN** `openspec/changes/fix-refund-rounding/proposal.md` has the three behaviour sections and the line `Kind: bug`

### Requirement: Bugfix changes are checked for a reproduction and a keep scenario
For a bug change, `sj spec check` SHALL warn when it has no keep scenario and
SHALL report an error when it has no normal (non-keep) scenario to reproduce the bug.

#### Scenario: SPEC-041 A bugfix with no keep scenario is warned about
- **WHEN** a bug change has one normal scenario and no keep scenario
- **THEN** `sj spec check` prints `warn: bug change has no [keep] scenario; add one for behaviour that must not change` and exits 0

#### Scenario: SPEC-042 A bugfix with only keep scenarios is an error
- **WHEN** a bug change has keep scenarios only
- **THEN** `sj spec check` exits 1 with `bug change has no reproduction scenario`
