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

### Requirement: sj ids can be limited to some capabilities
`sj ids [--write] [<target>...]` SHALL accept capability names and paths to spec files or
folders. A path SHALL select the capability of every open spec file under it, in `specs/` or in
an open change. The command SHALL list and write only the scenarios of the selected
capabilities: their `specs/` files and their delta copies in open changes. It selects whole
capabilities so that a MODIFIED copy and its original always get the same ID.

The prefix checks (IDS-005) SHALL cover every capability whose prefix a selected capability
uses, anywhere under the spec root, and numbering SHALL still cover the whole spec root; a
collision between two capabilities that aren't selected and don't share a selected prefix SHALL
NOT stop the run. A new
number SHALL continue from the highest one used with the prefix anywhere under the spec root.
A target that selects no capability SHALL exit 2, list the known capabilities, and write
nothing.

#### Scenario: IDS-014 Writing one capability leaves the others alone
- **WHEN** capabilities `staff` and `billing` both have scenarios without IDs, and `sj ids --write staff` runs
- **THEN** only the headings in `specs/staff/spec.md` change, `specs/billing/spec.md` is byte-identical, and the summary counts only `staff` scenarios

#### Scenario: IDS-015 A spec path limits the listing the same way
- **WHEN** `sj ids openspec/specs/staff/spec.md` runs in a project whose spec root is `openspec`
- **THEN** it lists only the scenarios without IDs in that file, and writes nothing

#### Scenario: IDS-016 An unknown target is refused
- **WHEN** `sj ids --write staf` runs and no capability or path is named `staf`
- **THEN** it exits 2, names `staf`, lists the known capabilities, and changes no file

#### Scenario: IDS-017 A target still stops on colliding prefixes
- **WHEN** capabilities `staff` and `st-aff` both produce `STAFF` and have scenarios without IDs, and `sj ids --write staff` runs
- **THEN** it changes no file and exits 2 naming `staff` and `st-aff`

#### Scenario: IDS-021 An unrelated collision does not stop a target
- **WHEN** capabilities `billing` and `bill-ing` both produce `BILLING`, `staff` has scenarios without IDs, and `sj ids --write staff` runs
- **THEN** it writes IDs in `specs/staff/spec.md` and exits 0

### Requirement: An original takes the ID its MODIFIED copy already has
When a `specs/` scenario without an ID has a copy in an open change's MODIFIED requirement, and
that copy already has an ID, `sj ids --write` SHALL give the original that ID, not a new number.
A copy matches when it has the same capability, requirement name and scenario name, ignoring IDs
and `[keep]`. A requirement renamed in the same change (`## RENAMED Requirements`) SHALL match
by its old name.

The original SHALL get a new number instead, and `sj ids` SHALL print a note line naming the
scenarios involved, in two cases:
- another scenario already uses that ID;
- copies in two open changes carry different IDs.

The note SHALL end by saying to give the delta copies the original's new ID.

`--json` SHALL carry the same note.

#### Scenario: IDS-018 The original takes the delta's ID
- **WHEN** a change's `## MODIFIED Requirements` › `### Requirement: Role changes are audited` has `#### Scenario: STAFF-057 Role change is recorded`, `specs/staff/spec.md` has `#### Scenario: Role change is recorded` without an ID, and the highest STAFF number in use is STAFF-099
- **THEN** `sj ids --write staff` changes that heading in `specs/staff/spec.md` to `#### Scenario: STAFF-057 Role change is recorded`, and the delta is unchanged

#### Scenario: IDS-019 Copies that disagree get a new number and a note
- **WHEN** open changes `c1` and `c2` both have a MODIFIED copy of `Role change is recorded`, one as STAFF-057 and one as STAFF-058, the original has no ID, and the highest STAFF number in use is STAFF-099
- **THEN** `sj ids --write staff` gives the original STAFF-100 and prints a note naming both copies, and neither delta changes

#### Scenario: IDS-020 A renamed requirement matches by its old name
- **WHEN** a change renames `Role changes are audited` to `Role changes are logged`, has a MODIFIED copy `STAFF-057 Role change is recorded` under the new name, and `specs/staff/spec.md` has `Role change is recorded` without an ID under the old name
- **THEN** `sj ids --write staff` gives the original STAFF-057
