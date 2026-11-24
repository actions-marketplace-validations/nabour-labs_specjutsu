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
When a `specs/` scenario without an ID has a copy in an open change's MODIFIED requirement
(same capability, requirement name and scenario name, ignoring IDs and `[keep]`), and that copy
already has an ID, `sj ids --write` SHALL give the original that ID, not a new number. If
another scenario already uses that ID, the original SHALL get a new number, and `sj ids` SHALL
print a line naming both scenarios.

#### Scenario: IDS-018 The original takes the delta's ID
- **WHEN** a change's `## MODIFIED Requirements` › `### Requirement: Role changes are audited` has `#### Scenario: STAFF-057 Role change is recorded`, `specs/staff/spec.md` has `#### Scenario: Role change is recorded` without an ID, and the highest STAFF number in use is STAFF-099
- **THEN** `sj ids --write staff` changes that heading in `specs/staff/spec.md` to `#### Scenario: STAFF-057 Role change is recorded`, and the delta is unchanged
