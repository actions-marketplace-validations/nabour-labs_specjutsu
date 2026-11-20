## ADDED Requirements

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
