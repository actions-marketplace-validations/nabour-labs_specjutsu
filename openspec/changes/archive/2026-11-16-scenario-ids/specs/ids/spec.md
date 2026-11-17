## Purpose

Give every scenario a stable, never-reused ID, so existing spec folders can be traced and gated without editing thousands of headings by hand.

## ADDED Requirements

### Requirement: sj ids lists scenarios without IDs
`sj ids` SHALL print one summary line with the number of scenarios without IDs, then one line per such scenario with the ID it would receive, its file and line. It SHALL exit 0 when every scenario has an ID and 1 otherwise. It SHALL write nothing.

#### Scenario: IDS-001 Listing shows the proposed IDs
- **WHEN** `specs/user-auth/spec.md` holds `#### Scenario: Valid registration` and `#### Scenario: Password too short`, and no USERAUTH IDs exist
- **THEN** `sj ids` prints `2 scenarios without IDs`, then `USERAUTH-001 Valid registration (…:N)` and `USERAUTH-002 Password too short (…:M)`, and exits 1

### Requirement: sj ids --write assigns IDs in place
`sj ids --write` SHALL insert the ID at the start of each scenario name without an ID, and SHALL change no other byte of any file. Numbers SHALL have at least three digits.

#### Scenario: IDS-002 Writing changes only the headings
- **WHEN** `sj ids --write` runs on a spec file with two scenarios without IDs and CRLF line endings
- **THEN** the two headings become `#### Scenario: USERAUTH-001 Valid registration` and `#### Scenario: USERAUTH-002 Password too short`, and a byte diff of the file shows only those insertions

#### Scenario: IDS-003 Running twice is a no-op
- **WHEN** `sj ids --write` runs a second time
- **THEN** no file changes and it prints `0 scenarios without IDs`

### Requirement: Prefixes come from the capability, or from config
A scenario's prefix SHALL be its capability folder's name, uppercased with non-letters removed, unless `[ids.prefixes]` in `specjutsu.toml` maps that capability to a prefix. A scenario that already has an ID keeps it. When two capabilities produce the same prefix, `sj ids --write` SHALL write nothing and exit 2, naming both capabilities.

#### Scenario: IDS-004 A configured prefix overrides the folder name
- **WHEN** `[ids.prefixes]` maps `user-auth = "AUTH"`
- **THEN** new IDs in `user-auth` are `AUTH-001`, `AUTH-002`, …

#### Scenario: IDS-005 Colliding prefixes stop the write
- **WHEN** capabilities `chat` and `c-hat` both produce `CHAT`
- **THEN** `sj ids --write` changes no file and exits 2 naming `chat` and `c-hat`

### Requirement: An ID is never reused
A new ID's number SHALL be one more than the highest number used with that prefix anywhere under the spec root, including `changes/archive/`.

#### Scenario: IDS-006 Numbers continue after archived IDs
- **WHEN** `specs/auth/spec.md` holds AUTH-001 and AUTH-002, and an archived change once held AUTH-007
- **THEN** the next new AUTH scenario gets AUTH-008

### Requirement: Delta copies share their spec's ID
A scenario without an ID in an open change's delta SHALL get the ID of the `specs/` scenario of the same capability with the same requirement name and scenario name, when one exists.

#### Scenario: IDS-007 A MODIFIED copy gets the original's ID
- **WHEN** `specs/auth/spec.md` has `### Requirement: Login` › `Valid login` and a change's `## MODIFIED Requirements` repeats `### Requirement: Login` › `Valid login` with a reworded THEN line, and both lack IDs
- **THEN** `sj ids --write` gives both the same ID
