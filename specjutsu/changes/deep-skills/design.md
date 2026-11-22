## Context

The ten `sj-*` skills total about 24.5 KB, and none is over 3.7 KB. SKILL-002 caps each body
at 6,000 bytes, and `claude::SKILLS` is `[(name, include_str!(SKILL.md)); 10]`, reused by
`shared_skills` for Codex and Gemini. So a skill can only ever be one short file. Superpowers
7.0.1 keeps a 120–330-line `SKILL.md` and moves the techniques and subagent prompts into
files next to it: `subagent-driven-development/{implementer,task-reviewer,re-review}-prompt.md`,
`systematic-debugging/{root-cause-tracing,defense-in-depth,condition-based-waiting}.md`,
`test-driven-development/writing-good-tests.md`. The agent reads those only at the step that
needs them.

`sj-apply` today: sequential tasks run in the main agent with no review; `[P]` batches go to
subagents and "When they finish, review each diff" is done by the main agent. The pilot names
this as the clearest gap.

## Goals / Non-Goals

**Goals:**
- Skills can ship reference files, loaded on demand, so `SKILL.md` stays short to read on
  every invocation.
- A fresh reviewer per task, a bounded fix loop and a final whole-change review in `sj-apply`.
- The debugging, TDD and verification techniques Superpowers teaches, adapted to `sj`
  commands.
- Rules in the skill check so this depth can't be dropped later without a failing test.

**Non-Goals:**
- Running or scoring the skills with a model. specjutsu never calls an LLM. `docs/skill-evals.md`
  describes manual pressure tests, Superpowers-style, to run before a release.
- Copying Superpowers text. The references are written from scratch around `sj next`,
  `sj test`, `sj verify` and the ledger, and say less where the ledger already enforces a rule.
- A reviewer subagent type or model choice. The skill says "a fresh subagent" and lets the
  agent tool decide how.

## Decisions

### 1. Reference files instead of a larger cap
A skill folder holds `SKILL.md` plus any `*.md` files; each file name in the folder is a
reference. `SKILL.md` stays at 6,000 bytes; references are at most 8,000 bytes each.
*Rejected:* raising the cap to ~15 KB. It loads everything on every invocation, and long
single files are where agents skim. Superpowers' own layout shows the split works.

### 2. How the check sees references
- `SKILL.md` names a reference by its file name in backticks or a Markdown link
  (`` `root-cause-tracing.md` `` or `[…](root-cause-tracing.md)`). Any bare lowercase
  `*.md` name it mentions (no `/` or `<` before it) must exist in the folder, except
  `principles.md`, `design.md`, `proposal.md`, `tasks.md` and `spec.md`, which name project
  files. Upper-case names such as `SKILL.md` and `CLAUDE.md` are never references.
- Every `*.md` in the folder other than `SKILL.md` must be named by `SKILL.md`.
- The foreign-command rule runs on references too.

| Rule | Message |
| --- | --- |
| named reference missing | `names missing reference <f>` |
| file in folder not named | `reference <f> is not named in SKILL.md` |
| reference over 8,000 bytes | `reference <f> is <n> bytes (limit 8000)` |

### 3. The review loop in sj-apply, and what the check matches
The loop, for every task:
1. Main agent runs `sj next <change>` and gives its output, plus `implementer-prompt.md`, to a
   new implementer subagent. The implementer does red, green and refactor with `sj test`, and
   commits; it never runs `sj task done`. It reports `DONE`, `DONE_WITH_CONCERNS`, `BLOCKED`
   or `NEEDS_CONTEXT`.
2. Main agent gives a fresh subagent `task-reviewer-prompt.md`, the scenario text from
   `sj next`, the commit range, and nothing from the implementer's report. The reviewer runs
   `git diff` and `sj verify <change>`, checks the test asserts the THEN with the scenario's
   values and would fail without the change, checks scope and principles, and answers
   `APPROVED` or `CHANGES_REQUESTED` with `file:line` findings.
3. On `CHANGES_REQUESTED`, the findings go back to the implementer; then a fresh subagent with
   `re-review-prompt.md` checks only those findings and any new diff. After 3 rounds without
   approval, stop and ask the user.
4. On `APPROVED`, the main agent runs `sj task done`.
After the last task, a fresh subagent runs `sj-review` over the whole change, then `sj-verify`.

`[P]` batches run step 1 in parallel; each task still gets its own reviewer.

The check matches fixed phrases, like the onboard check does, so the wording is a contract:

| Skill / file | Must contain | Message when missing |
| --- | --- | --- |
| sj-apply | `implementer-prompt.md`, `task-reviewer-prompt.md`, `re-review-prompt.md` | `does not name <f>` |
| sj-apply | `fresh reviewer subagent` | `does not send each task to a fresh reviewer subagent` |
| sj-apply | `3 rounds` | `does not limit the fix loop to 3 rounds` |
| task-reviewer-prompt.md | `sj verify` | `task-reviewer-prompt.md does not name sj verify` |
| task-reviewer-prompt.md | `Don't trust the implementer's report` | `task-reviewer-prompt.md does not say to ignore the implementer's report` |
| task-reviewer-prompt.md | `git diff`, `APPROVED`, `CHANGES_REQUESTED`, `fail without the change` | `task-reviewer-prompt.md does not name <x>` |
| implementer-prompt.md | `sj test`, the four statuses; must NOT contain `sj task done` except in `never run \`sj task done\`` | `implementer-prompt.md runs sj task done` / `does not name <x>` |
| sj-apply, sj-debug, sj-verify | `## Rationalizations`, `## Red flags` | `has no ## <x> section` |
| sj-apply | `writing-good-tests.md` | `does not name writing-good-tests.md` |
| sj-debug | the three reference names; `3 failed fixes` | `does not name <f>` / `does not question the design after 3 failed fixes` |
| sj-verify | `a subagent's report is not evidence` | `does not say a subagent's report is not evidence` |

Phrases are matched case-insensitively with whitespace collapsed, so line wrapping doesn't
break them.
*Rejected:* only checking that the reference files exist. A file can exist and say nothing;
the phrases pin the behaviours the pilot asked for.

### 4. Coverage depth column
`COVERAGE.md` becomes `| Tool | Item | Skill | Depth |`. Depth is a file name relative to the
skill folder. The parser still accepts the 3-column form for OpenSpec and Spec Kit rows
(empty depth). Planned depth for the Superpowers rows:

| Item | Skill | Depth |
| --- | --- | --- |
| brainstorming | sj-explore | SKILL.md |
| writing-plans | sj-propose | SKILL.md |
| executing-plans | sj-apply | SKILL.md |
| test-driven-development | sj-apply | writing-good-tests.md |
| subagent-driven-development | sj-apply | task-reviewer-prompt.md |
| dispatching-parallel-agents | sj-apply | SKILL.md |
| systematic-debugging | sj-debug | root-cause-tracing.md |
| requesting-code-review | sj-review | SKILL.md |
| receiving-code-review | sj-review | SKILL.md |
| verification-before-completion | sj-verify | SKILL.md |
| finishing-a-development-branch | sj-finish | SKILL.md |
| using-git-worktrees | sj-finish | SKILL.md |

### 5. Shipping references
`claude::SKILLS` keeps its `(name, body)` shape, so no caller changes. A new
`claude::REFERENCES: [(skill, file, body); N]` lists every reference, each an `include_str!`;
a test fails when an embedded reference is missing from it.
*Rejected:* a `Skill { name, body, refs }` struct. It changes every caller of `SKILLS` for
no behaviour gain.
`skills_plan` writes each reference with the same
`generated()` marker (an HTML comment at the end, as `SKILL.md` has); `is_unmodified`,
`--force` and the keep path apply unchanged. `stale_skills` also lists marked, unmodified
`*.md` files in an `sj-*` folder that the current `SKILLS` doesn't ship. `skills_remove` removes
each unmodified reference with its `SKILL.md` before `remove_if_empty`, so ADAPT-010's
install-then-remove still restores every byte once references ship. `shared_skills` and the global install
reuse the same functions, so Codex, Gemini and `--global` get references for free.
`tests/assets.rs` compares every file in each `skills/sj-*` folder with its embedded copy,
and `scripts/sync-adapter-assets.sh` copies every `*.md` in each skill folder.

## Risks / Trade-offs

- [Phrase checks make wording rigid] → Phrases are short, listed in one table here, and the
  messages name the missing phrase.
- [More subagents per task cost more tokens] → The skill lets a task with no scenarios
  (`no-scenario`) skip the reviewer, and the re-reviewer checks only the findings. Users who
  want the old behaviour can edit the installed skill; `sj update` keeps the edit.
- [References not read by agents that ignore links] → `SKILL.md` names each reference at the
  step that needs it ("Give the reviewer `task-reviewer-prompt.md`"), not in a list at the end.
- [Hand-edited installs] → Existing ADAPT-004 behaviour: modified files are reported and kept.
