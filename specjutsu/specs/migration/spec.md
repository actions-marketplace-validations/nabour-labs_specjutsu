# migration Specification

## Purpose
Move a project off OpenSpec, Spec Kit and Superpowers onto specjutsu in one previewable command, keeping all spec content and every project-owned setting.

## Requirements

### Requirement: Migration previews by default and needs a clean tree to write
`sj migrate --from <tool>…` SHALL print every file it would create, change or remove, and SHALL write nothing. With `--write` it SHALL refuse, exiting 2, when the working tree has uncommitted changes. When it finishes it SHALL print `git diff --stat`-style counts and the rollback command `sj migrate --undo`.

#### Scenario: MIG-001 [keep] Preview changes nothing
- **WHEN** `sj migrate --from openspec --from superpowers` runs without `--write`
- **THEN** it lists the planned changes, and `git status --porcelain` is empty afterwards

#### Scenario: MIG-002 [keep] A dirty tree blocks the write
- **WHEN** `sj migrate --from openspec --write` runs with an unstaged edit to `README.md`
- **THEN** nothing changes and it exits 2 asking to commit or stash first

### Requirement: Migrating from OpenSpec keeps all specs and history
With `--from openspec`, the migration SHALL keep `openspec/` (specs, open changes and archive) as the spec root and assign IDs with `sj ids --write`. It SHALL move `openspec/config.yaml`'s `context`, `rules` and `operations` guidance to `<spec_root>/principles.md`, and remove OpenSpec's generated agent files (skills and commands whose metadata names OpenSpec as generator). Other files under `.claude/` SHALL be kept.

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

### Requirement: Migrating from OpenSpec removes its Antigravity workflows
With `--from openspec`, the migration SHALL also remove each Antigravity workflow OpenSpec generated: `<agent>/workflows/opsx-<name>.md` whose frontmatter holds only a `description` and whose body runs the `openspec` CLI. Every other workflow, including an `opsx-*` file that never runs `openspec`, SHALL be kept.

#### Scenario: MIG-008 OpenSpec's Antigravity workflows are removed
- **WHEN** a project has `.agents/workflows/opsx-propose.md` and `.agents/workflows/opsx-apply.md`, each with only a `description` in its frontmatter and a body that runs `openspec status`, a project workflow `.agents/workflows/deploy.md`, and `.agents/workflows/opsx-notes.md` with only a `description` and a body that never runs the `openspec` CLI, and `sj migrate --from openspec --write` runs
- **THEN** the preview lists `remove .agents/workflows/opsx-propose.md (OpenSpec workflow)` and the same for `opsx-apply.md`, both files are gone afterwards, and `deploy.md` and `opsx-notes.md` are unchanged and not listed

### Requirement: Migration installs adapters for the agents a project already uses
Besides the Claude adapter, the migration SHALL install every adapter `sj init --agents auto` would install for the agent files the project has, and the preview SHALL list each of their files.

#### Scenario: MIG-010 Detected agents get their adapters
- **WHEN** a project has `AGENTS.md` and `.cursor/` but no `.claude/`, and `sj migrate --from openspec --write` runs
- **THEN** the preview lists `change AGENTS.md (adapter)` and the Cursor hook file with `(adapter)`, and afterwards `AGENTS.md` holds the specjutsu block, the Cursor pre-edit hook is installed, and the Claude adapter is installed

### Requirement: A migration commit passes the gate
`sj migrate --write` SHALL add `.specjutsu/ledger/` to `.gitignore` as `sj adopt` does, its
preview SHALL list `change .gitignore (ignore the ledger)`, or `create .gitignore (ignore the
ledger)` when there is none, whenever it will, and the commit of its changes SHALL pass
`sj gate` without an exemption.

#### Scenario: MIG-011 The migration commit passes the gate
- **WHEN** an OpenSpec project with a root `pnpm-lock.yaml`, a vitest package, `.claude/settings.json` and a `.gitignore` without the ledger runs `sj migrate --from openspec --write`, and the result is committed
- **THEN** the preview listed `change .gitignore (ignore the ledger)`, `.gitignore` holds `.specjutsu/ledger/`, the vitest suite has `setup = "pnpm install --frozen-lockfile"`, and `sj gate --base <the commit before>` exits 0

### Requirement: Migrating from OpenSpec pins short prefixes before assigning IDs
With `--from openspec`, the migration SHALL map each capability that has no IDs and no mapping to the prefix `sj ids prefixes` suggests. It SHALL write that mapping into the `[ids.prefixes]` table of `specjutsu.toml` before assigning IDs.

#### Scenario: MIG-012 OpenSpec capabilities get short prefixes
- **WHEN** an OpenSpec project has capability `distribution` whose scenarios have no IDs, and `sj migrate --from openspec --write` runs
- **THEN** `specjutsu.toml` maps `distribution = "DIST"`, and the first scenario in `distribution` is headed `#### Scenario: DIST-001 …`

### Requirement: The preview can show each file's diff, and says hooks are merged
With `--diff`, the preview SHALL print a unified diff (`--- a/<path>`, `+++ b/<path>`, `@@` hunks) for every file it would create or change. A file it would create is diffed against nothing. The preview line for `.claude/settings.json` SHALL state how many existing hook entries it keeps.

#### Scenario: MIG-013 --diff shows the settings.json change and writes nothing
- **WHEN** `.claude/settings.json` holds a Superpowers `enabledPlugins` entry and one project PreToolUse hook running `npm run lint`, and `sj migrate --from superpowers --diff` runs
- **THEN** the output holds a diff for `.claude/settings.json` in which every `-` line is part of the Superpowers entry or only JSON punctuation (brackets, braces and commas), every `+` line is part of a specjutsu hook entry or only JSON punctuation, no `-` or `+` line contains `npm run lint`, and `git status --porcelain` is empty afterwards

#### Scenario: MIG-014 The preview says existing hooks are kept
- **WHEN** `.claude/settings.json` holds one project PreToolUse hook, and `sj migrate --from superpowers` runs
- **THEN** the line for `.claude/settings.json` ends with `(merged: 1 existing hook kept)`

### Requirement: sj migrate --undo puts back only what migrate touched
`sj migrate --write` SHALL record the commit it started from and every file it created, changed or removed, with each file's content hash after the write and its content before the write, in `sj-migrate-undo.json` inside the git directory (`git rev-parse --git-dir`), so it is never committed. `sj migrate --undo` SHALL:
- restore every changed or removed file to its content before the write, including files git ignores;
- delete every file migrate created;
- delete the manifest;
- touch no other file.

It SHALL refuse, exiting 2 and changing nothing, when any recorded file differs from its recorded hash, naming each such file. With no manifest it SHALL exit 2 and say there is nothing to undo.

#### Scenario: MIG-017 A write ends with the undo rollback line
- **WHEN** `sj migrate --from openspec --write` finishes
- **THEN** its last line is `rollback: sj migrate --undo`, and no line mentions `git clean`

#### Scenario: MIG-015 Undo restores migrate's files and keeps new ones
- **WHEN** `sj migrate --from openspec --write` ran, the user then created the untracked file `notes.txt`, and `sj migrate --undo` runs
- **THEN** every file migrate changed or removed has its pre-migration content, the files it created are gone, `notes.txt` still exists, and the undo manifest is gone

#### Scenario: MIG-016 Undo refuses when a migrated file was edited since
- **WHEN** `sj migrate --from superpowers --write` ran, `.claude/settings.json` was then edited by hand, and `sj migrate --undo` runs
- **THEN** it exits 2 naming `.claude/settings.json`, and no file changes

#### Scenario: MIG-018 Undo restores a gitignored file migrate changed
- **WHEN** `.claude/settings.json` is gitignored and holds one project hook, `sj migrate --from superpowers --write` runs, and then `sj migrate --undo` runs
- **THEN** `.claude/settings.json` has exactly its pre-migration content
