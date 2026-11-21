# authoring Specification

## Purpose
Create, check and archive spec changes from the command line, so a project needs no other spec tool to run a spec-driven workflow.

## Requirements

### Requirement: sj spec new scaffolds a change
`sj spec new <change>` SHALL create `<spec_root>/changes/<change>/` with `proposal.md`, `design.md`, `tasks.md` and an empty `specs/` directory, each from a template. `<change>` MUST be kebab-case. It SHALL refuse an existing change name, and print the created path and the next free ID for every prefix in use.

#### Scenario: SPEC-001 A new change is scaffolded
- **WHEN** `sj spec new add-oauth-login` runs and AUTH-001…AUTH-004 exist
- **THEN** the change folder holds the three templates and `specs/`, and the output names the folder and `next AUTH-005`

#### Scenario: SPEC-002 Bad or duplicate names are refused
- **WHEN** `sj spec new Add_OAuth` runs, or `sj spec new add-oauth-login` runs a second time
- **THEN** nothing is created and it exits 2 with the reason

### Requirement: sj spec list shows open changes
`sj spec list` SHALL print one line per open change: its name, checked and total tasks, and the number of scenarios it adds or modifies. `--json` SHALL print the same data.

#### Scenario: SPEC-003 List shows progress
- **WHEN** change `add-oauth-login` has 3 of 7 tasks checked and adds 4 scenarios
- **THEN** `sj spec list` prints `add-oauth-login  3/7 tasks  4 scenarios`

### Requirement: sj spec validate checks structure
`sj spec validate` SHALL report, with file and line: a `#### Scenario:` without an ID, a duplicate ID, a scenario with no `WHEN` or no `THEN` step, a requirement with no scenario or no `SHALL`/`MUST`, a delta file whose requirements are outside `## ADDED|MODIFIED|REMOVED|RENAMED Requirements`, a REMOVED requirement without `**Reason**` and `**Migration**`, a MODIFIED or REMOVED requirement whose name matches no requirement in `specs/`, and a task line that names no scenario ID and is not marked `(no-scenario: <reason>)`. It SHALL exit 0 when clean and 1 otherwise. `--strict` SHALL also fail on warnings.

#### Scenario: SPEC-004 A scenario without THEN is reported
- **WHEN** a delta scenario AUTH-005 has a WHEN line and no THEN line
- **THEN** validate prints `AUTH-005 has no THEN step (<file>:<line>)` and exits 1

#### Scenario: SPEC-005 MODIFIED must name an existing requirement
- **WHEN** a delta has `## MODIFIED Requirements` › `### Requirement: Login flow` and `specs/auth/spec.md` has no requirement named `Login flow`
- **THEN** validate reports `MODIFIED requirement not found: Login flow`

#### Scenario: SPEC-006 A task without a scenario ID is reported
- **WHEN** `tasks.md` has `- [ ] 2.1 Wire up the router` with no ID and no `(no-scenario: …)` marker
- **THEN** validate reports that task with its line

### Requirement: sj spec check finds gaps between spec and tasks
`sj spec check <change>` SHALL report every scenario the change adds or modifies that no task names, every task naming an ID the change and `specs/` do not define, and every `[NEEDS CLARIFICATION: <question>]` marker in the change's files. It SHALL exit 0 when there are none and 1 otherwise.

#### Scenario: SPEC-007 An uncovered scenario and an open question are reported
- **WHEN** the change adds AUTH-005 and AUTH-006, tasks name only AUTH-005, and `proposal.md` contains `[NEEDS CLARIFICATION: token lifetime?]`
- **THEN** check prints `uncovered AUTH-006` and `clarify token lifetime? (proposal.md:<line>)` and exits 1

### Requirement: sj spec archive applies the delta and moves the change
`sj spec archive <change>` SHALL, in this order: refuse with exit 1 if validate fails or any task is unchecked, unless `--force`. Then apply the delta to `<spec_root>/specs/<capability>/spec.md`: ADDED requirements are appended, MODIFIED ones replace the requirement of the same name, REMOVED ones are deleted, and RENAMED ones change the name. A new capability's spec is created with the delta's `## Purpose`. Finally, move the change to `changes/archive/<YYYY-MM-DD>-<change>/`. The merged file SHALL be what `openspec archive` writes: LF line endings, one blank line between requirement blocks, and runs of blank lines outside fenced code collapsed to one. A spec already in that layout SHALL keep every byte outside the edited requirement blocks.

#### Scenario: SPEC-008 Archive merges all four delta kinds
- **WHEN** a change adds requirement `Refresh`, modifies `Login`, removes `Legacy token` and renames `Logout` to `Sign out` in capability `auth`
- **THEN** `specs/auth/spec.md` has `Refresh` appended, `Login` replaced, `Legacy token` gone and `Sign out` in place of `Logout`, and the change folder is under `changes/archive/` with today's date

#### Scenario: SPEC-009 Archive refuses open tasks
- **WHEN** `sj spec archive add-oauth-login` runs with 1 unchecked task
- **THEN** nothing changes and it exits 1 naming the open task

#### Scenario: SPEC-010 A new capability gets its Purpose
- **WHEN** a change adds capability `billing` whose delta starts with `## Purpose` text
- **THEN** archive creates `specs/billing/spec.md` with that Purpose and the added requirements

#### Scenario: SPEC-011 An archived OpenSpec change is readable unchanged
- **WHEN** a change created by OpenSpec (with `.openspec.yaml`) is archived by `sj spec archive`
- **THEN** the result matches what `openspec archive` produces for the same input, ignoring the metadata file

### Requirement: sj spec check keeps inline code in open questions
`sj spec check <change>` SHALL print each `clarify` question with the text of its inline-code spans intact. A marker that itself starts inside an inline-code span SHALL still be ignored.

#### Scenario: SPEC-012 Clarify questions keep their inline code
- **WHEN** `design.md` contains a clarify marker whose question is "do `PreToolUse` hooks fire for `apply_patch` file edits?"
- **THEN** check prints `clarify do PreToolUse hooks fire for apply_patch file edits? (design.md:<line>)`, with the inline code text kept (backticks may be kept)

### Requirement: Templates can be overridden per project
`sj templates` SHALL list each template (`proposal.md`, `design.md`, `tasks.md`, `spec.md`) with the path it resolves to: `<spec_root>/templates/<name>` when that file exists, otherwise `built-in:<name>`. `sj spec new` and `sj instructions` SHALL use the same resolution. `--json` SHALL print an array of objects with `name`, `path` and `source` (`project` or `built-in`).

#### Scenario: SPEC-034 Templates list built-ins by default
- **WHEN** `<spec_root>/templates/` does not exist
- **THEN** `sj templates` prints four lines, each naming a template and `built-in:<name>`

#### Scenario: SPEC-035 A project template wins and is used by sj spec new
- **WHEN** `openspec/templates/proposal.md` exists with the heading `## Problem`
- **THEN** `sj templates` lists `proposal.md  openspec/templates/proposal.md`, and `sj spec new add-x` writes a `proposal.md` starting with `## Problem`

#### Scenario: SPEC-036 JSON templates name their source
- **WHEN** only `openspec/templates/tasks.md` is overridden and `sj templates --json` runs
- **THEN** the `tasks.md` entry has `source` = `project` and the other three have `source` = `built-in`

### Requirement: sj spec show prints one spec, change or scenario
`sj spec show <item>` SHALL resolve `<item>` in this order: a scenario ID, an open change name, an archived change (by its `changes/archive/` folder name, or by its bare change name when exactly one archived change has it), a capability name. An archived change SHALL print as an open one does, with `archived <YYYY-MM-DD>` in place of the task count; a bare name matching several archived changes SHALL exit 1 and list their folder names. For a capability it SHALL print `<spec_root>/specs/<capability>/spec.md` as it is on disk. For a change it SHALL print the proposal, each delta file under a header naming its capability, and the change's checked and total tasks. For a scenario it SHALL print the capability, the requirement name and its SHALL sentence, the scenario's steps, each mapped test as `<file>:<line> <name>`, and the latest ledger result with whether red-before-green is proven. `--json` SHALL print the same data as one JSON object with a `kind` field of `capability`, `change` or `scenario`. When `<item>` matches nothing, it SHALL exit 1 and list up to 3 known names or IDs whose edit distance is lowest. Scenarios in an open change's delta SHALL be found as well as those in `specs/`.

#### Scenario: SPEC-013 Show a scenario with its evidence
- **WHEN** AUTH-003 is in `specs/auth/spec.md`, test `auth_003_rejects_expired_token` at `tests/auth.rs:12` names it, and the ledger shows a failed then a passed run
- **THEN** `sj spec show AUTH-003` prints the requirement name, the WHEN and THEN lines, `tests/auth.rs:12 auth_003_rejects_expired_token` and `latest passed, red-before-green proven`, and exits 0

#### Scenario: SPEC-014 Show a change
- **WHEN** open change `add-oauth-login` has a proposal, a delta for `auth` and 3 of 7 tasks checked
- **THEN** `sj spec show add-oauth-login` prints the proposal text, a header naming `auth` followed by that delta, and `3/7 tasks`

#### Scenario: SPEC-015 Show a capability byte for byte
- **WHEN** `specs/auth/spec.md` exists and no change or scenario ID is named `auth`
- **THEN** `sj spec show auth` prints that file's contents unchanged

#### Scenario: SPEC-016 JSON names the kind
- **WHEN** `sj spec show AUTH-003 --json` runs
- **THEN** stdout is one JSON object with `"kind": "scenario"`, `"id": "AUTH-003"`, a `tests` array and a `latest` field

#### Scenario: SPEC-017 Unknown item suggests near matches
- **WHEN** capabilities `auth` and `audit` exist and `sj spec show aut` runs
- **THEN** it exits 1 and the output names `auth` and `audit` as suggestions

#### Scenario: SPEC-018 A scenario in an open change is found
- **WHEN** AUTH-009 exists only in `changes/add-oauth-login/specs/auth/spec.md` and has no test
- **THEN** `sj spec show AUTH-009` prints its steps, names the change `add-oauth-login`, and reports `untested`

#### Scenario: SPEC-019 Show an archived change by name
- **WHEN** `changes/archive/2026-11-18-audit-checks/` exists, no open change is named `audit-checks`, and `sj spec show audit-checks` runs
- **THEN** it prints that change's proposal and deltas with `archived 2026-11-18`, and `sj spec show 2026-11-18-audit-checks` prints the same

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
