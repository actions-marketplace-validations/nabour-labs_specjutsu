# Tasks

Each task group lands as two commits: the failing tests first (`test(IDs): red`), then the code (`feat(IDs): green`). Tests go under each crate's `tests/` directory so the gate can overlay them on base.

## 1. The skill check (sj-adapters)

- [x] 1.1 `sj_adapters::skills::check` with every rule in design decision 2 (SKILL-001..004). Verify: `crates/sj-adapters/tests/skills.rs` builds fixture skill folders and sees each message: a missing mapping, a duplicate, an unknown skill, a 7,000-byte `sj-propose`, an `sj-apply` that runs a foreign status command, a missing required command, and a review skill without the principles rule.

## 2. The skills (skills/)

- [x] 2.1 The nine `skills/sj-*/SKILL.md` files and `skills/COVERAGE.md` (design decisions 1, 3) (SKILL-001..004). Verify: the test that checks this repo's `skills/` passes, and every body is at most 6,000 bytes.

## 3. Docs and self-hosting

- [x] 3.1 README section on the skills, and the CHANGELOG entry (no-scenario: docs). Verify: built `sj gate --base origin/main` passes and `sj spec check workflow-skills` is clean.
