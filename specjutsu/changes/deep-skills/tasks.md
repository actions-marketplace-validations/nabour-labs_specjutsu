# Tasks

Each task lands as two commits: the failing tests first (`test(IDs): red`), then the code
(`feat(IDs): green`). Tests go in `crates/sj-adapters/tests/` so the gate can overlay them on
base. A task that adds a check rule also updates the shipped skills it applies to, and their
copies under `crates/sj-adapters/assets/skills/`, so this repo's skills check clean after
every task.

## 1. Reference files in the skill check (sj-adapters)

- [x] 1.1 Reference rules in `skills::check`: missing, unnamed and oversized references, and foreign commands in references (design decision 2) (SKILL-002, SKILL-010)

## 2. sj-apply review loop

- [x] 2.1 sj-apply loop rules and the implementer and task-reviewer prompt rules (design decision 3); rewrite `skills/sj-apply/SKILL.md` around the per-task review loop and write `implementer-prompt.md`, `task-reviewer-prompt.md` and `re-review-prompt.md` (SKILL-011..013)

## 3. Shipping references (sj-adapters)

- [ ] 3.1 `Skill { name, body, refs }` in `claude::SKILLS`; `skills_plan` installs references for Claude and the shared `.agents/skills`; `tests/assets.rs` compares every file in each skill folder (design decision 5) (ADAPT-045)
- [ ] 3.2 [P] Hand-edited references are kept on `sj update`, replaced with `--force` (ADAPT-046)
- [ ] 3.3 [P] Stale references removed on update; `--remove-agents` removes references and empty folders (ADAPT-047)

## 4. Discipline depth

- [ ] 4.1 Discipline-section rule and the sj-debug and sj-verify phrase rules; write `skills/sj-apply/writing-good-tests.md` with `## Rationalizations` and `## Red flags` in sj-apply; rewrite `skills/sj-debug/SKILL.md` in four phases with `root-cause-tracing.md`, `defense-in-depth.md` and `condition-based-waiting.md`; add the gate procedure, rationalizations and red flags to `skills/sj-verify/SKILL.md` (SKILL-014..016)

## 5. Coverage depth

- [ ] 5.1 `COVERAGE.md` `Depth` column: parse 4 columns, require depth for Superpowers rows, check the file exists, and fill the depths from design decision 4 (SKILL-017)

## 6. Docs

- [ ] 6.1 README skills section, `docs/migration-readiness.md` (Superpowers depth row), CHANGELOG entry (no-scenario: docs)
- [ ] 6.2 `docs/skill-evals.md`: manual pressure tests for sj-apply, sj-debug and sj-verify, and the steps to re-run the pilot (no-scenario: docs)
