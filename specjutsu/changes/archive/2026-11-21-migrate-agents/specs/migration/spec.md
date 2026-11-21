## ADDED Requirements

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
