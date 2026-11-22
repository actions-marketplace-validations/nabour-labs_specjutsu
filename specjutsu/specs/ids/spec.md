# ids Specification

## Purpose
Give every scenario a stable, never-reused ID, so existing spec folders can be traced and gated without editing thousands of headings by hand.

## Requirements

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

### Requirement: sj ids prefixes suggests a short prefix for every capability
`sj ids prefixes` SHALL print an `[ids.prefixes]` TOML table with one line per capability folder, sorted, and write nothing. The prefix it gives is chosen in this order:
1. A prefix mapped in `[ids.prefixes]` is kept.
2. A capability whose scenarios already carry IDs keeps the prefix those IDs use.
3. Otherwise the suggestion is the capability's letters, uppercased. Names of six letters or fewer are kept whole, and longer ones are cut to their first four letters.

When two suggestions are equal, or a suggestion equals a prefix already kept, the suggestions SHALL be lengthened together, one letter at a time, until every prefix is distinct. When a clash remains after every name has run out of letters, `sj ids prefixes` SHALL exit 2 naming the clashing capabilities and their prefix, and `--write` SHALL write nothing.

#### Scenario: IDS-008 Long capability names get four-letter prefixes
- **WHEN** capability folders `distribution`, `gate` and `payment-methods` exist, none of their scenarios has an ID, there is no `[ids.prefixes]`, and `sj ids prefixes` runs
- **THEN** it prints `distribution = "DIST"`, `gate = "GATE"` and `payment-methods = "PAYM"` under `[ids.prefixes]`, exits 0, and no file changes

#### Scenario: IDS-009 Prefixes already in use are pinned
- **WHEN** `user-auth` scenarios carry `USERAUTH-001` and `USERAUTH-002`, `[ids.prefixes]` maps `billing = "BILL"`, and `sj ids prefixes` runs
- **THEN** it prints `billing = "BILL"` and `user-auth = "USERAUTH"`

#### Scenario: IDS-010 Clashing suggestions lengthen until distinct
- **WHEN** capability folders `transfers` and `transactions` have no IDs and no mapping, and `sj ids prefixes` runs
- **THEN** it prints `transactions = "TRANSA"` and `transfers = "TRANSF"`

#### Scenario: IDS-013 An unresolvable clash is refused
- **WHEN** `[ids.prefixes]` maps `billing = "BILL"`, capability folders `bill` and `billing` exist with no IDs, and `sj ids prefixes --write` runs
- **THEN** it exits 2 naming `bill` and `billing` and the prefix `BILL`, and `specjutsu.toml` is unchanged

### Requirement: sj ids prefixes --write adds the table to specjutsu.toml
With `--write`, `sj ids prefixes` SHALL add every capability missing from `[ids.prefixes]` to that table in `specjutsu.toml`, creating the table at the end of the file if it is absent. It SHALL leave every other byte unchanged. Running it twice SHALL change nothing the second time.

#### Scenario: IDS-011 Writing keeps the rest of the file
- **WHEN** `specjutsu.toml` holds a comment line, `mode = "strict"` and `[ids.prefixes]` with `billing = "BILL"`, capability `distribution` has no mapping, and `sj ids prefixes --write` runs twice
- **THEN** after the first run the file is the same apart from one added line `distribution = "DIST"` under `[ids.prefixes]`, and the second run leaves it unchanged

### Requirement: sj ids hints when a derived prefix is long
When `sj ids` would assign a prefix derived from a folder name longer than six letters, its listing SHALL end with a hint naming the capability, the long prefix and the suggested one.

#### Scenario: IDS-012 A long derived prefix gets a hint
- **WHEN** capability `distribution` has a scenario without an ID, has no mapping, and `sj ids` runs
- **THEN** the listing ends with `hint: distribution would use DISTRIBUTION; sj ids prefixes --write maps it to DIST`
