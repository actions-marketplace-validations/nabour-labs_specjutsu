## MODIFIED Requirements

### Requirement: Migration previews by default and needs a clean tree to write
`sj migrate --from <tool>…` SHALL print every file it would create, change or remove, and SHALL write nothing. With `--write` it SHALL refuse, exiting 2, when the working tree has uncommitted changes. When it finishes it SHALL print `git diff --stat`-style counts and the rollback command `sj migrate --undo`.

#### Scenario: MIG-001 Preview changes nothing
- **WHEN** `sj migrate --from openspec --from superpowers` runs without `--write`
- **THEN** it lists the planned changes, and `git status --porcelain` is empty afterwards

#### Scenario: MIG-002 A dirty tree blocks the write
- **WHEN** `sj migrate --from openspec --write` runs with an unstaged edit to `README.md`
- **THEN** nothing changes and it exits 2 asking to commit or stash first

## ADDED Requirements

### Requirement: Migrating from OpenSpec pins short prefixes before assigning IDs
With `--from openspec`, the migration SHALL map each capability that has no IDs and no mapping to the prefix `sj ids prefixes` suggests. It SHALL write that mapping into the `[ids.prefixes]` table of `specjutsu.toml` before assigning IDs.

#### Scenario: MIG-012 OpenSpec capabilities get short prefixes
- **WHEN** an OpenSpec project has capability `distribution` whose scenarios have no IDs, and `sj migrate --from openspec --write` runs
- **THEN** `specjutsu.toml` maps `distribution = "DIST"`, and the first scenario in `distribution` is headed `#### Scenario: DIST-001 …`

### Requirement: The preview can show each file's diff, and says hooks are merged
With `--diff`, the preview SHALL print a unified diff (`--- a/<path>`, `+++ b/<path>`, `@@` hunks) for every file it would create or change. A file it would create is diffed against nothing. The preview line for `.claude/settings.json` SHALL state how many existing hook entries it keeps.

#### Scenario: MIG-013 --diff shows the settings.json change and writes nothing
- **WHEN** `.claude/settings.json` holds a Superpowers `enabledPlugins` entry and one project PreToolUse hook running `npm run lint`, and `sj migrate --from superpowers --diff` runs
- **THEN** the output holds a diff for `.claude/settings.json` whose `-` lines are only the Superpowers entry and whose `+` lines are only the specjutsu hook entries, the `npm run lint` hook is not on a `-` line, and `git status --porcelain` is empty afterwards

#### Scenario: MIG-014 The preview says existing hooks are kept
- **WHEN** `.claude/settings.json` holds one project PreToolUse hook, and `sj migrate --from superpowers` runs
- **THEN** the line for `.claude/settings.json` ends with `(merged: 1 existing hook kept)`

### Requirement: sj migrate --undo puts back only what migrate touched
`sj migrate --write` SHALL record the commit it started from and every file it created, changed or removed, with each file's content hash after the write, in `sj-migrate-undo.json` inside the git directory (`git rev-parse --git-dir`), so it is never committed. `sj migrate --undo` SHALL:
- restore every changed or removed file to its content at that commit;
- delete every file migrate created;
- delete the manifest;
- touch no other file.

It SHALL refuse, exiting 2 and changing nothing, when any recorded file differs from its recorded hash, naming each such file. With no manifest it SHALL exit 2 and say there is nothing to undo.

#### Scenario: MIG-015 Undo restores migrate's files and keeps new ones
- **WHEN** `sj migrate --from openspec --write` ran, the user then created the untracked file `notes.txt`, and `sj migrate --undo` runs
- **THEN** every file migrate changed or removed has its pre-migration content, the files it created are gone, `notes.txt` still exists, and the undo manifest is gone

#### Scenario: MIG-016 Undo refuses when a migrated file was edited since
- **WHEN** `sj migrate --from superpowers --write` ran, `.claude/settings.json` was then edited by hand, and `sj migrate --undo` runs
- **THEN** it exits 2 naming `.claude/settings.json`, and no file changes
