## ADDED Requirements

### Requirement: sj spec archive matches MODIFIED scenarios without the keep marker
When `sj spec archive` checks that a MODIFIED requirement keeps every scenario of the current
spec, it SHALL compare scenario headings with any `[keep]` marker removed, so a scenario
carried into a change as `[keep]` matches the same scenario without the marker. The merged
spec SHALL contain the MODIFIED block as the delta writes it, marker included.

#### Scenario: SPEC-043 A keep marker in a MODIFIED block matches the unmarked scenario
- **WHEN** `specs/auth/spec.md` has `#### Scenario: AUTH-002 Wrong password` under `Login`, and a change's `## MODIFIED Requirements` › `### Requirement: Login` has `#### Scenario: AUTH-002 [keep] Wrong password`
- **THEN** archive succeeds and `specs/auth/spec.md` has `#### Scenario: AUTH-002 [keep] Wrong password` under `Login`

#### Scenario: SPEC-044 [keep] A MODIFIED block that drops a scenario is still refused
- **WHEN** a change's `## MODIFIED Requirements` › `### Requirement: Login` has `AUTH-001 Valid login` but not `AUTH-002 Wrong password`
- **THEN** archive fails with `current spec contains scenario(s) not present in the modified block: "AUTH-002 Wrong password"`
