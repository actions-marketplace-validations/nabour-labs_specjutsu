## Why

A pilot project reported that the `sj-*` skills can't replace Superpowers yet. They are
50–90-line checklists, while Superpowers' test-driven-development, systematic-debugging,
verification-before-completion and subagent-driven-development skills are 120–330 lines and
come with reference files and prompt templates that the agent loads only when it needs them.
The clearest gap is in `sj-apply`: Superpowers sends each task to a fresh reviewer subagent
that never saw the implementer's work, but `sj-apply` has the main agent review the parallel
diffs itself, the same agent that dispatched them.

The cause is structural. SKILL-002 caps each skill body at 6,000 bytes, and the adapters ship
exactly one file per skill (`claude::SKILLS` embeds only `SKILL.md`). `skills/COVERAGE.md`
maps each Superpowers skill to an `sj-*` skill but doesn't check that the techniques are
there, so the check never saw the gap. We said specjutsu replaces Superpowers
(migration-readiness finding 5), so the skills have to carry the depth.

## What Changes

- A skill folder may ship reference files (`*.md` next to `SKILL.md`). `SKILL.md` keeps its
  6,000-byte cap and names each reference. Each reference has its own 8,000-byte cap. The skill
  check fails on a missing, unnamed or oversized reference.
- The adapters install, update and remove reference files the same way as `SKILL.md`: a
  generated marker, hand edits kept unless `--force`, and removal of unmodified files.
- `sj-apply` sends every task, sequential or `[P]`, to an implementer subagent, then to a fresh
  task-reviewer subagent that checks the scenario, the diff and `sj verify`, not the
  implementer's report. The fix loop goes to a fresh re-reviewer each round, for at most three
  rounds before asking the user. Only the main agent runs `sj task done`, after approval. The
  change ends with a whole-change `sj-review` in a fresh subagent. New references:
  `implementer-prompt.md`, `task-reviewer-prompt.md`, `re-review-prompt.md`.
- `sj-apply` covers the TDD judgment the ledger can't check, in `writing-good-tests.md`.
- `sj-debug` gets four phases, the three-failed-fixes rule, and the references
  `root-cause-tracing.md`, `defense-in-depth.md` and `condition-based-waiting.md`.
- `sj-verify` gets a gate procedure, and the rule that a subagent's report is not evidence.
- `sj-apply`, `sj-debug` and `sj-verify` each get `## Rationalizations` and `## Red flags`
  sections.
- `skills/COVERAGE.md` gets a fourth column, `Depth`, naming the file that covers each
  Superpowers item. The check fails if that file doesn't exist in that skill.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `skills`: reference files, the per-task review loop, discipline sections and coverage depth.
- `adapters`: install, update and remove reference files.

## Impact

- `crates/sj-adapters/src/skills.rs`: reference, review-loop, discipline and depth rules.
- `crates/sj-adapters/src/claude.rs` and `shared_skills.rs`: `SKILLS` grows a list of
  reference files per skill; `skills_plan`, `skills_remove` and stale-file cleanup handle them.
- `skills/sj-apply`, `skills/sj-debug`, `skills/sj-verify`, `skills/sj-review`,
  `skills/COVERAGE.md`, and their copies under `crates/sj-adapters/assets/skills/`.
- `crates/sj-adapters/tests/assets.rs`: compares reference files too.
- Docs: README skills section, `docs/migration-readiness.md`, CHANGELOG, and a manual
  pressure-test guide in `docs/skill-evals.md`.
- No new dependencies. No CLI flags change.
