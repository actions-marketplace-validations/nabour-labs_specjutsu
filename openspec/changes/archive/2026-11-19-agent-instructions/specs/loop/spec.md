## ADDED Requirements

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
