## ADDED Requirements

### Requirement: sj ids can be limited to some capabilities
`sj ids [--write] [<target>...]` SHALL accept capability names and paths to spec files or
folders. A path SHALL select the capability of every open spec file under it, in `specs/` or in
an open change. The command SHALL list and write only the scenarios of the selected
capabilities: their `specs/` files and their delta copies in open changes. It selects whole
capabilities so that a MODIFIED copy and its original always get the same ID.

The prefix checks (IDS-005) and the numbering SHALL still cover the whole spec root. A new
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
