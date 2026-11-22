## ADDED Requirements

### Requirement: Adapters ship a skill's reference files
Every adapter that installs the `sj-*` skills SHALL install each skill's reference files next to its `SKILL.md`, with the same generated marker. `sj update` SHALL treat a reference file like `SKILL.md`: rewrite it when unmodified, report and keep it when hand-edited unless `--force`, and delete an unmodified reference that the new version no longer ships. `sj init --remove-agents` SHALL remove unmodified reference files and the skill folder when it is left empty.

#### Scenario: ADAPT-045 Init installs the reference files
- **WHEN** `sj init --agents claude,codex` runs in an empty repository
- **THEN** `.claude/skills/sj-apply/task-reviewer-prompt.md` and `.agents/skills/sj-apply/task-reviewer-prompt.md` exist, each byte-identical to the shipped reference plus the generated marker, and a second `sj update` prints `adapters up to date`

#### Scenario: ADAPT-046 A hand-edited reference is kept on update
- **WHEN** a user edited `.claude/skills/sj-apply/task-reviewer-prompt.md` and runs `sj update`
- **THEN** the file is unchanged and the output names it as modified, and `sj update --force` replaces it

#### Scenario: ADAPT-047 A reference no longer shipped is removed
- **WHEN** `.claude/skills/sj-debug/old-technique.md` carries the generated marker, is unmodified, and the new version doesn't ship it, and `sj update` runs
- **THEN** the file is deleted and the output names it as removed; after `sj init --remove-agents claude` no `.claude/skills/sj-*` folder remains
