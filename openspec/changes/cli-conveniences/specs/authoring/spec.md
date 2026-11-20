## ADDED Requirements

### Requirement: sj spec archive archives several changes at once
`sj spec archive` SHALL accept one or more change names, or `--all-done` for every open change whose tasks are all checked. It SHALL validate every named change and check every precondition of the single-change archive before writing anything; if any change fails, it SHALL archive none, name each failing change with its reason, and exit 1. Changes SHALL be applied in the order given (alphabetical for `--all-done`). Two changes in one run that MODIFY, REMOVE or RENAME the same requirement in the same capability SHALL be refused, naming both changes and the requirement. `--all-done` with no finished change SHALL print `nothing to archive` and exit 0.

#### Scenario: SPEC-030 Archive several finished changes
- **WHEN** `sj spec archive add-login add-refresh` runs and both changes validate with all tasks checked
- **THEN** both deltas are merged into `specs/`, both folders move to `changes/archive/` with today's date, and it exits 0

#### Scenario: SPEC-031 One failing change archives nothing
- **WHEN** `sj spec archive add-login add-refresh` runs and `add-refresh` has 1 unchecked task
- **THEN** no file under `specs/` or `changes/` changes, the output names `add-refresh` and its open task, and it exits 1

#### Scenario: SPEC-032 Two changes editing one requirement are refused
- **WHEN** `add-login` and `add-sso` both have `## MODIFIED Requirements` › `### Requirement: Login flow` in capability `auth` and are archived in one run
- **THEN** nothing changes, the output names `add-login`, `add-sso` and `auth: Login flow`, and it exits 1

#### Scenario: SPEC-033 All-done archives only finished changes
- **WHEN** `sj spec archive --all-done` runs with 3 open changes, of which `add-login` and `add-refresh` have every task checked and `add-sso` has 2 unchecked tasks
- **THEN** `add-login` and `add-refresh` are archived, `add-sso` stays open, and the output lists the two archived changes
