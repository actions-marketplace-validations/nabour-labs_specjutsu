## Purpose

Move a project off OpenSpec, Spec Kit and Superpowers onto specjutsu in one previewable command, keeping all spec content and every project-owned setting.

## ADDED Requirements

### Requirement: Migration previews by default and needs a clean tree to write
`sj migrate --from <tool>…` SHALL print every file it would create, change or remove, and SHALL write nothing. With `--write` it SHALL refuse, exiting 2, when the working tree has uncommitted changes. When it finishes it SHALL print `git diff --stat`-style counts and the rollback command.

#### Scenario: MIG-001 Preview changes nothing
- **WHEN** `sj migrate --from openspec --from superpowers` runs without `--write`
- **THEN** it lists the planned changes, and `git status --porcelain` is empty afterwards

#### Scenario: MIG-002 A dirty tree blocks the write
- **WHEN** `sj migrate --from openspec --write` runs with an unstaged edit to `README.md`
- **THEN** nothing changes and it exits 2 asking to commit or stash first

### Requirement: Migrating from OpenSpec keeps all specs and history
With `--from openspec`, the migration SHALL keep `openspec/` (specs, open changes and archive) as the spec root and assign IDs with `sj ids --write`. It SHALL move `openspec/config.yaml`'s `context` to `<spec_root>/principles.md` and its `rules` to `specjutsu.toml`, and remove OpenSpec's generated agent files (skills and commands whose metadata names OpenSpec as generator). Other files under `.claude/` SHALL be kept.

#### Scenario: MIG-003 OpenSpec agent files are removed, spec content kept
- **WHEN** a project has `openspec/specs/` with 3,242 scenarios, `.claude/skills/openspec-*/` (6 skills), `.claude/commands/opsx/` and a project skill `.claude/skills/deploy/`
- **THEN** after `--write`, every scenario has an ID and its text is otherwise unchanged, the 6 skills and `opsx` commands are gone, and `.claude/skills/deploy/` is unchanged

### Requirement: Migrating from Spec Kit imports, then removes Spec Kit files
With `--from speckit`, the migration SHALL run the Spec Kit import, and only after it succeeds remove `.specify/`, each `specs/NNN-*/` folder it imported, and Spec Kit's agent command files. A failed import SHALL remove nothing.

#### Scenario: MIG-004 Spec Kit files go only after a good import
- **WHEN** `sj migrate --from speckit --write` runs in `chrome-agent`
- **THEN** `openspec/specs/chrome-ai-agent/spec.md` and its archived change exist, `.specify/` and `specs/001-chrome-ai-agent/` are removed, and `sj spec validate` reports no errors other than listed import gaps

#### Scenario: MIG-005 A failed import removes nothing
- **WHEN** the import fails because a target path exists
- **THEN** `.specify/` and `specs/` are unchanged and the command exits 2

### Requirement: Migrating from Superpowers keeps project settings
With `--from superpowers`, the migration SHALL remove from `.claude/settings.json` the `superpowers@…` entry in `enabledPlugins`, the Superpowers marketplace in `extraKnownMarketplaces`, and permission rules naming `superpowers:` skills or `docs/superpowers/`. Every other key, hook and permission SHALL be kept in order. Files under `docs/superpowers/` SHALL be kept and listed for the user to review.

#### Scenario: MIG-006 Project hooks survive Superpowers removal
- **WHEN** `.claude/settings.json` enables Superpowers, denies three `Skill(superpowers:…)` rules and `Edit(/docs/superpowers/**)`, and runs `require-worktree.mjs` and `clean-merged-branches.mjs` hooks
- **THEN** after `--write` the plugin, marketplace and four deny rules are gone, both hooks are unchanged, and the specjutsu hooks are added

### Requirement: Every migration ends installed and checked
After the tool-specific steps, the migration SHALL write `specjutsu.toml` with `sj adopt` when it is missing, install the Claude adapter, and run `sj doctor` and `sj spec validate`, printing their results. It SHALL exit 0 when both are clean and 1 otherwise, keeping the changes for the user to fix or revert.

#### Scenario: MIG-007 A migrated project is ready to work
- **WHEN** `sj migrate --from openspec --from superpowers --write` completes in `FridgeSense`
- **THEN** `specjutsu.toml` lists the `backend` pytest suite and the `pwa` vitest suite, the adapter is installed, `sj doctor` and `sj spec validate` print no problems, and `sj gate --base main` on the migration commit passes
