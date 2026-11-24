## ADDED Requirements

### Requirement: sj spec validate points MODIFIED scenarios at sj ids
When a scenario in a MODIFIED requirement matches, by name without its ID and `[keep]` marker,
a scenario without an ID in the same requirement in `specs/`, `sj spec validate` SHALL say to
run `sj ids --write <capability>` first, so both specs use the same name. When the delta
scenario has an ID this SHALL be a warning; when it has none, the hint SHALL be appended to the
missing-ID error. When the delta scenario and an original with the same name both have IDs
and the IDs differ, `sj spec validate` SHALL warn that the names differ for tools that match
by full name, and say to give the delta the original's ID.

#### Scenario: SPEC-045 An ID the original lacks is a warning with the fix
- **WHEN** a delta's `## MODIFIED Requirements` › `### Requirement: Role changes are audited` has `#### Scenario: STAFF-057 Role change is recorded`, and `specs/staff/spec.md` has that requirement with `#### Scenario: Role change is recorded`
- **THEN** validate warns `STAFF-057 Role change is recorded: specs/staff/spec.md has this scenario without an ID; run sj ids --write staff first (<file>:<line>)`, exits 0, and exits 1 with `--strict`

#### Scenario: SPEC-046 A missing ID in a MODIFIED copy names the same fix
- **WHEN** the same delta scenario is `#### Scenario: Role change is recorded`, without an ID
- **THEN** validate's missing-ID error for it ends with `; run sj ids --write staff first`, and it exits 1

#### Scenario: SPEC-047 Different IDs on a copy and its original are a warning
- **WHEN** a delta's MODIFIED copy is `#### Scenario: STAFF-057 Role change is recorded` and `specs/staff/spec.md` has `#### Scenario: STAFF-100 Role change is recorded` in the same requirement
- **THEN** validate warns `STAFF-057 Role change is recorded: specs/staff/spec.md has this scenario as STAFF-100; give the delta copy STAFF-100 (<file>:<line>)`, exits 0, and exits 1 with `--strict`
