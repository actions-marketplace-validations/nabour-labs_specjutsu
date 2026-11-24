## ADDED Requirements

### Requirement: sj ids can be limited to some capabilities
`sj ids [--write] [<target>...]` SHALL accept capability names and paths to spec files or
folders. It SHALL list and write only the scenarios in those targets, plus their delta copies
in open changes. New numbers SHALL still continue from the highest number used with the prefix
anywhere under the spec root. A target that names no capability and no existing path SHALL
exit 2, list the known capabilities, and write nothing.

#### Scenario: IDS-014 Writing one capability leaves the others alone
- **WHEN** capabilities `staff` and `billing` both have scenarios without IDs, and `sj ids --write staff` runs
- **THEN** only the headings in `specs/staff/spec.md` change, `specs/billing/spec.md` is byte-identical, and the summary counts only `staff` scenarios

#### Scenario: IDS-015 A spec path limits the listing the same way
- **WHEN** `sj ids openspec/specs/staff/spec.md` runs in a project whose spec root is `openspec`
- **THEN** it lists only the scenarios without IDs in that file, and writes nothing

#### Scenario: IDS-016 An unknown target is refused
- **WHEN** `sj ids --write staf` runs and no capability or path is named `staf`
- **THEN** it exits 2, names `staf`, lists the known capabilities, and changes no file
