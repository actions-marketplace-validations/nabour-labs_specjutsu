## MODIFIED Requirements

### Requirement: Prefixes come from the capability, or from config
A scenario's prefix SHALL be its capability folder's name, uppercased with non-letters removed, unless `[ids.prefixes]` in `specjutsu.toml` maps that capability to a prefix. A scenario that already has an ID keeps it. When two capabilities produce the same prefix, `sj ids --write` SHALL write nothing and exit 2, naming both capabilities. This holds when one of them already has every scenario numbered, as long as the other still needs IDs.

#### Scenario: IDS-004 [keep] A configured prefix overrides the folder name
- **WHEN** `[ids.prefixes]` maps `user-auth = "AUTH"`
- **THEN** new IDs in `user-auth` are `AUTH-001`, `AUTH-002`, …

#### Scenario: IDS-005 [keep] Colliding prefixes stop the write
- **WHEN** capabilities `chat` and `c-hat` both produce `CHAT`
- **THEN** `sj ids --write` changes no file and exits 2 naming `chat` and `c-hat`

#### Scenario: IDS-023 A numbered capability's prefix is taken
- **WHEN** every scenario in `specs/staff/spec.md` has an ID (STAFF-001), `specs/st-aff/spec.md` has scenarios without IDs, and both capabilities produce `STAFF`
- **THEN** `sj ids --write` and `sj ids --write st-aff` each change no file and exit 2 naming `staff` and `st-aff`

### Requirement: Delta copies share their spec's ID
A scenario without an ID in an open change's delta SHALL get the ID of the `specs/` scenario of the same capability with the same requirement name and scenario name, when one exists. A requirement renamed in the same change (`## RENAMED Requirements`) SHALL match by its old name.

#### Scenario: IDS-007 [keep] A MODIFIED copy gets the original's ID
- **WHEN** `specs/auth/spec.md` has `### Requirement: Login` › `Valid login` and a change's `## MODIFIED Requirements` repeats `### Requirement: Login` › `Valid login` with a reworded THEN line, and both lack IDs
- **THEN** `sj ids --write` gives both the same ID

#### Scenario: IDS-022 A copy under a renamed requirement gets the original's ID
- **WHEN** a change renames `Role changes are audited` to `Role changes are logged`, has a MODIFIED copy `Role change is recorded` without an ID under the new name, `specs/staff/spec.md` has `Role change is recorded` without an ID under the old name, and the highest STAFF number in use is STAFF-099
- **THEN** `sj ids --write staff` gives both STAFF-100
