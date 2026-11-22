## MODIFIED Requirements

### Requirement: Skills are small and well-formed
Each skill SHALL be a `SKILL.md` with `name` and `description` frontmatter, a description that says when to use it, and a body of at most 6,000 bytes. A skill folder MAY also hold reference files, `*.md` files next to `SKILL.md`, each at most 8,000 bytes. `SKILL.md` SHALL name every reference file in its folder, and every `*.md` file name that `SKILL.md` names SHALL exist in the folder. Skills and reference files SHALL NOT name `openspec`, `specify`, `/speckit` or `superpowers:` commands.

#### Scenario: SKILL-002 An oversized or foreign skill fails the check
- **WHEN** `sj-propose` grows to 7,000 bytes, or `sj-apply` tells the agent to run `openspec status`
- **THEN** the skill check fails naming the skill and the rule

#### Scenario: SKILL-010 A missing, unnamed or oversized reference fails the check
- **WHEN** `sj-debug/SKILL.md` names `root-cause-tracing.md` but the folder has no such file, the folder holds `notes.md` that `SKILL.md` never names, and the folder holds `defense-in-depth.md` of 9,000 bytes
- **THEN** the skill check reports `sj-debug: names missing reference root-cause-tracing.md`, `sj-debug: reference notes.md is not named in SKILL.md` and `sj-debug: reference defense-in-depth.md is 9000 bytes (limit 8000)`

## ADDED Requirements

### Requirement: sj-apply reviews every task with a fresh subagent
`sj-apply` SHALL send each task, sequential or `[P]`, to an implementer subagent using `implementer-prompt.md`, then to a fresh task-reviewer subagent using `task-reviewer-prompt.md`. Findings SHALL go back to the implementer and then to a fresh re-reviewer using `re-review-prompt.md`, for at most 3 rounds before the agent asks the user. Only the main agent SHALL run `sj task done`, and only after the reviewer approves. When every task is done, `sj-apply` SHALL run `sj-review` over the whole change in a fresh subagent.

#### Scenario: SKILL-011 sj-apply names the per-task review loop
- **WHEN** the skill check reads an `sj-apply` that has the main agent review the diffs itself, names no `task-reviewer-prompt.md` or `re-review-prompt.md`, and gives no round limit
- **THEN** the check reports `sj-apply: does not name task-reviewer-prompt.md`, `sj-apply: does not name re-review-prompt.md`, `sj-apply: does not send each task to a fresh reviewer subagent` and `sj-apply: does not limit the fix loop to 3 rounds`, and the shipped `sj-apply` reports none of them

#### Scenario: SKILL-012 The task reviewer works from evidence, not the implementer's report
- **WHEN** the skill check reads a `task-reviewer-prompt.md` that passes on the implementer's summary and never names `sj verify`
- **THEN** the check reports `sj-apply: task-reviewer-prompt.md does not name sj verify` and `sj-apply: task-reviewer-prompt.md does not say to ignore the implementer's report`; the shipped prompt names the scenario text, `git diff`, `sj verify`, the question whether the test fails without the change, and the verdicts `APPROVED` and `CHANGES_REQUESTED`

#### Scenario: SKILL-013 The implementer reports a status and never closes the task
- **WHEN** the skill check reads an `implementer-prompt.md` that tells the implementer to run `sj task done`
- **THEN** the check reports `sj-apply: implementer-prompt.md runs sj task done`; the shipped prompt names `sj test` and the statuses `DONE`, `DONE_WITH_CONCERNS`, `BLOCKED` and `NEEDS_CONTEXT`

### Requirement: Discipline skills carry their techniques
`sj-apply`, `sj-debug` and `sj-verify` SHALL each have a `## Rationalizations` section and a `## Red flags` section. `sj-apply` SHALL name `writing-good-tests.md`. `sj-debug` SHALL name `root-cause-tracing.md`, `defense-in-depth.md` and `condition-based-waiting.md`, and SHALL tell the agent to question the design after 3 failed fixes. `sj-verify` SHALL tell the agent that a subagent's report is not evidence.

#### Scenario: SKILL-014 A discipline skill without its sections fails the check
- **WHEN** the skill check reads an `sj-debug` with no `## Red flags` heading and an `sj-verify` with no `## Rationalizations` heading
- **THEN** the check reports `sj-debug: has no ## Red flags section` and `sj-verify: has no ## Rationalizations section`

#### Scenario: SKILL-015 sj-debug names its techniques and the three-fix rule
- **WHEN** the skill check reads an `sj-debug` that names neither `condition-based-waiting.md` nor a limit of 3 failed fixes
- **THEN** the check reports `sj-debug: does not name condition-based-waiting.md` and `sj-debug: does not question the design after 3 failed fixes`

#### Scenario: SKILL-016 sj-verify does not trust a subagent's report
- **WHEN** the skill check reads an `sj-verify` that never says a subagent's report is not evidence
- **THEN** the check reports `sj-verify: does not say a subagent's report is not evidence`

### Requirement: Coverage names where each technique lives
Each Superpowers row in `skills/COVERAGE.md` SHALL have a `Depth` column naming a file in its skill's folder, `SKILL.md` or a reference file, that covers the item. Rows for OpenSpec and Spec Kit MAY leave it empty.

#### Scenario: SKILL-017 A depth file that doesn't exist fails the check
- **WHEN** the Superpowers row `systematic-debugging` maps to `sj-debug` with depth `root-cause.md`, which `sj-debug/` doesn't have, and the row `verification-before-completion` has an empty depth
- **THEN** the check reports `coverage: Superpowers systematic-debugging depth sj-debug/root-cause.md does not exist` and `coverage: Superpowers verification-before-completion has no depth`
