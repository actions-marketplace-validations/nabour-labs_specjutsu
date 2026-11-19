# loop Specification

## Purpose
Answer the agent's working-loop questions (where am I, what next, am I done) from files and test evidence, in a few lines, so the agent never has to read whole spec folders or raw test logs.

## Requirements

### Requirement: sj status summarises each open change
`sj status` SHALL print at most 10 lines per open change: its name, checked and total tasks, and its added or modified scenarios counted as proven (red-before-green in the ledger, latest green), green-unproven, red, and untested. It SHALL end with the next unchecked task. `sj status <change>` SHALL show only that change. `--json` SHALL print the same data.

#### Scenario: LOOP-001 Status fits in ten lines
- **WHEN** change `add-oauth-login` has 7 tasks (3 checked) and 4 scenarios: 2 proven, 1 red, 1 untested
- **THEN** `sj status add-oauth-login` prints at most 10 lines including `3/7 tasks`, `2 proven, 1 red, 1 untested` and `next 2.1 <task text>`

### Requirement: sj next gives the minimal context for one task
`sj next [<change>]` SHALL print, for the first unchecked task: the task line, the full text of each scenario it names, the paths of files changed since the merge-base, the path of `<spec_root>/principles.md` when it exists, and one `sj test <ID>` command per scenario. The output SHALL be at most 1,600 bytes plus the scenario texts.

#### Scenario: LOOP-002 Next shows one task and its scenarios
- **WHEN** the first unchecked task is `- [ ] 2.1 Refresh tokens (AUTH-005, AUTH-006)`
- **THEN** `sj next` prints that line, the WHEN/THEN steps of AUTH-005 and AUTH-006, and `sj test AUTH-005` and `sj test AUTH-006`, and no other task

#### Scenario: LOOP-003 Parallel tasks are listed only when independent
- **WHEN** open tasks 3.1 [P] (AUTH-007) and 3.2 [P] (AUTH-008) link to tests in different files, and 3.3 [P] (AUTH-009) links to a test in the same file as AUTH-007
- **THEN** `sj next --parallel` lists 3.1 and 3.2 as one batch and 3.3 separately

### Requirement: Task completion can require proof
`sj task done <change> <task-number>` SHALL check that task's box in `tasks.md` and change no other byte. In `strict` mode it SHALL refuse, exiting 1 and naming each unproven ID, unless the ledger shows red-before-green and a latest green result for every scenario the task names. In `guided` and `advisory` modes it SHALL check the box and print a warning for each unproven ID.

#### Scenario: LOOP-004 Strict mode refuses an unproven task
- **WHEN** mode is `strict` and task 2.1 names AUTH-005, which has only `passed` records in the ledger
- **THEN** `sj task done add-oauth-login 2.1` leaves `tasks.md` unchanged and prints `unproven AUTH-005 (no red run recorded)`, exiting 1

#### Scenario: LOOP-005 Guided mode warns but completes
- **WHEN** mode is `guided` with the same ledger
- **THEN** the box is checked and a warning names AUTH-005

### Requirement: sj verify is the local check before a PR
`sj verify [<change>]` SHALL run the tests linked to the change's added and modified scenarios, and report failing tests, orphans, `sj spec check` problems, unproven scenarios (per the ledger), and the gate's unspecified-change and exemption rules against the merge-base with the default branch. It SHALL NOT run red-on-base. It SHALL print `verify: ok` and exit 0 when there are no problems, and one line per problem with exit 1 otherwise.

#### Scenario: LOOP-006 Verify catches what the gate would catch
- **WHEN** the change adds AUTH-005 with a passing test and AUTH-006 with no test
- **THEN** `sj verify` prints `untested AUTH-006` and exits 1

#### Scenario: LOOP-007 Verify passes a finished change
- **WHEN** every added scenario has a passing test proven red-before-green and `sj spec check` is clean
- **THEN** `sj verify` prints `verify: ok` and exits 0

### Requirement: sj instructions tells an agent what to write next
`sj instructions <change> [<artifact>]` SHALL print, for one artifact of the change (`proposal`, `specs`, `design` or `tasks`): the resolved template path, the project context, the rules for that artifact, the next free ID for every prefix in use, the project language when one is set, and the command that checks the artifact. With no `<artifact>` it SHALL pick the first of `proposal`, `specs`, `design`, `tasks` that is missing or still identical to its template. `--json` SHALL print the same data with the keys `change`, `artifact`, `template`, `context`, `rules`, `next_ids`, `language` and `check`. It SHALL exit 2 for an unknown change or artifact.

#### Scenario: LOOP-012 Instructions pick the first unwritten artifact
- **WHEN** change `add-oauth-login` has a written `proposal.md`, an empty `specs/` and template-only `design.md` and `tasks.md`
- **THEN** `sj instructions add-oauth-login` prints instructions for `specs` and names `changes/add-oauth-login/specs/<capability>/spec.md` as the file to write

#### Scenario: LOOP-013 JSON instructions carry every field
- **WHEN** `sj instructions add-oauth-login design --json` runs
- **THEN** it prints one JSON object with `change` = `add-oauth-login`, `artifact` = `design`, and the keys `template`, `context`, `rules`, `next_ids`, `language` and `check`, with `language` null when none is set

#### Scenario: LOOP-014 Instructions name the next free IDs
- **WHEN** AUTH-001…AUTH-004 exist in `specs/` and the open change `add-sso` adds AUTH-005
- **THEN** `sj instructions add-oauth-login specs` lists `next AUTH-006`

#### Scenario: LOOP-015 Instructions include only the artifact's own rules
- **WHEN** `principles.md` has `### tasks` and `### design` sections under `## Rules`
- **THEN** `sj instructions add-oauth-login tasks` prints the `tasks` rules and not the `design` rules

#### Scenario: LOOP-016 Each artifact names its check
- **WHEN** `sj instructions add-oauth-login <artifact>` runs for `specs`, then `tasks`
- **THEN** the check is `sj spec validate add-oauth-login` for `specs` and `sj spec check add-oauth-login` for `tasks`

#### Scenario: LOOP-017 Unknown change or artifact is an error
- **WHEN** `sj instructions no-such-change` runs, or `sj instructions add-oauth-login readme` runs
- **THEN** it prints nothing to stdout and exits 2, naming the unknown change or listing the four artifacts

#### Scenario: LOOP-018 A fully written change points to the loop
- **WHEN** all four artifacts of `add-oauth-login` are written and `sj instructions add-oauth-login` runs with no artifact
- **THEN** it prints `all artifacts written; run sj next add-oauth-login` and exits 0

#### Scenario: LOOP-019 Instructions carry the project language
- **WHEN** `specjutsu.toml` sets `language = "fr"`
- **THEN** `sj instructions add-oauth-login proposal` says to write prose in `fr` and to keep `### Requirement:`, `#### Scenario:`, `WHEN`, `THEN`, `SHALL`, `MUST` and the delta section headers in English
