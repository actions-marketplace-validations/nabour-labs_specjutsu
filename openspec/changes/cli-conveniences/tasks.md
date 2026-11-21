# Tasks

Each task group lands as two commits: the failing tests first (`test(IDs): red`), then the code (`feat(IDs): green`).

## 1. Bulk archive

- [x] 1.1 Split `sj-core::spec::archive` into plan and apply, with no change to single-change behaviour; accept several names (SPEC-030, SPEC-031).
- [x] 1.2 Conflict detection across changes in one run (SPEC-032).
- [x] 1.3 `--all-done` (SPEC-033).

## 2. Update preview

- [x] 2.1 [P] Split `sj-adapters` update into `plan()` and `apply()`; `--dry-run` with write, keep and remove lines (ADAPT-036..038).
- [x] 2.2 `--dry-run --json` (ADAPT-039).

## 3. Completions

- [x] 3.1 [P] `sj completions <shell>` with `clap_complete` (CONF-023..025).

## 4. Onboarding skill

- [ ] 4.1 [P] `skills/sj-onboard/SKILL.md`, the skill set and `COVERAGE.md` mapping OpenSpec `onboard` (SKILL-001, SKILL-005..007).

## 5. Docs

- [ ] 5.1 README and manual: bulk archive, `update --dry-run`, completions setup per shell, `sj-onboard`; CHANGELOG entry (no-scenario: docs).
