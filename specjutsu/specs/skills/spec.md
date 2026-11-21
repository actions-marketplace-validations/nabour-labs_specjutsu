# skills Specification

## Purpose
Provide one lean set of agent skills that covers spec-driven work from idea to merge, built on `sj` commands, replacing Superpowers, OpenSpec and Spec Kit skills and commands.

## Requirements

### Requirement: The skill set covers every replaced workflow
specjutsu SHALL ship the skills `sj-explore`, `sj-propose`, `sj-clarify`, `sj-principles`, `sj-apply`, `sj-debug`, `sj-review`, `sj-verify`, `sj-finish` and `sj-onboard`. `skills/COVERAGE.md` SHALL map every Superpowers skill (except `writing-skills` and `using-superpowers`), every OpenSpec workflow command, and every Spec Kit command to exactly one of them.

#### Scenario: SKILL-001 Coverage is complete
- **WHEN** the coverage check reads `skills/COVERAGE.md` and the lists of Superpowers skills, OpenSpec commands (explore, propose, apply, archive, sync, update, onboard) and Spec Kit commands (constitution, specify, clarify, plan, tasks, analyze, checklist, implement)
- **THEN** every listed item maps to one existing `sj-*` skill, and the check fails naming any item that is missing

### Requirement: Skills are small and well-formed
Each skill SHALL be a `SKILL.md` with `name` and `description` frontmatter, a description that says when to use it, and a body of at most 6,000 bytes. Skills SHALL NOT name `openspec`, `specify`, `/speckit` or `superpowers:` commands.

#### Scenario: SKILL-002 An oversized or foreign skill fails the check
- **WHEN** `sj-propose` grows to 7,000 bytes, or `sj-apply` tells the agent to run `openspec status`
- **THEN** the skill check fails naming the skill and the rule

### Requirement: Skills drive work through sj commands
`sj-apply` SHALL take each task from `sj next`, write the failing test first, and run `sj test <ID>` to record red, then green, before `sj task done`. `sj-debug` SHALL reproduce a bug as a failing test linked to a scenario before any fix. `sj-verify` SHALL run `sj verify` and treat any problem line as not done. `sj-finish` SHALL run `sj verify`, then `sj spec archive`, then open the PR. `sj-propose` SHALL end with `sj spec validate` and `sj spec check` clean. `sj-clarify` SHALL resolve each `[NEEDS CLARIFICATION]` marker that `sj spec check` reports.

#### Scenario: SKILL-003 Each skill names its required commands
- **WHEN** the skill check reads the skills
- **THEN** `sj-apply` names `sj next`, `sj test` and `sj task done`; `sj-debug` names `sj test`; `sj-verify` and `sj-finish` name `sj verify`; `sj-finish` names `sj spec archive`; `sj-propose` names `sj spec new`, `sj spec validate` and `sj spec check`; and `sj-clarify` names `sj spec check`

### Requirement: Principles are part of planning and review
`sj-propose` and `sj-review` SHALL read `<spec_root>/principles.md` when it exists, and record any deviation from a principle, with its reason, in the change's `design.md`. `sj-principles` SHALL create or amend that file.

#### Scenario: SKILL-004 Planning and review reference the principles file
- **WHEN** the skill check reads `sj-propose` and `sj-review`
- **THEN** both name `principles.md` and the design-deviation rule

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
