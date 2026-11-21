## MODIFIED Requirements

### Requirement: The skill set covers every replaced workflow
specjutsu SHALL ship the skills `sj-explore`, `sj-propose`, `sj-clarify`, `sj-principles`, `sj-apply`, `sj-debug`, `sj-review`, `sj-verify`, `sj-finish` and `sj-onboard`. `skills/COVERAGE.md` SHALL map every Superpowers skill (except `writing-skills` and `using-superpowers`), every OpenSpec workflow command, and every Spec Kit command to exactly one of them.

#### Scenario: SKILL-001 Coverage is complete
- **WHEN** the coverage check reads `skills/COVERAGE.md` and the lists of Superpowers skills, OpenSpec commands (explore, propose, apply, archive, sync, update, onboard) and Spec Kit commands (constitution, specify, clarify, plan, tasks, analyze, checklist, implement)
- **THEN** every listed item maps to one existing `sj-*` skill, and the check fails naming any item that is missing

## ADDED Requirements

### Requirement: sj-onboard walks a first change end to end
`sj-onboard` SHALL guide a new user through one complete change on a toy scenario in a scratch change named `sj-onboard-demo`: `sj spec new`, one scenario, `sj test <ID>` red, the smallest code to make it green, `sj test <ID>` green, `sj task done`, and `sj verify`. It SHALL explain each step in one or two sentences before running it. At the end it SHALL remove the scratch change folder, its test and its code, and SHALL NOT run `sj spec archive` on it, so `specs/` is unchanged. It SHALL refuse to start when a change named `sj-onboard-demo` already exists or the working tree has uncommitted changes, and say why.

#### Scenario: SKILL-005 Onboard names the full loop
- **WHEN** the skill check reads `skills/sj-onboard/SKILL.md`
- **THEN** it names `sj spec new`, `sj test`, `sj task done` and `sj verify` in that order, and the scratch change `sj-onboard-demo`

#### Scenario: SKILL-006 Onboard cleans up after itself
- **WHEN** the skill check reads `skills/sj-onboard/SKILL.md`
- **THEN** it tells the agent to delete `changes/sj-onboard-demo/` and the demo test and code at the end, and it does not tell the agent to run `sj spec archive`

#### Scenario: SKILL-007 Onboard refuses a dirty start
- **WHEN** the skill check reads `skills/sj-onboard/SKILL.md`
- **THEN** it tells the agent to stop when `git status` shows uncommitted changes or `changes/sj-onboard-demo/` exists, and it passes the size and naming rules of SKILL-002
