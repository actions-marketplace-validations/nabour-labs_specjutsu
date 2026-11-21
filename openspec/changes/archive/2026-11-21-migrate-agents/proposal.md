## Why

A dry run of `sj migrate --from openspec --from superpowers` on a production monorepo that
uses Claude Code and Antigravity found two gaps. OpenSpec's Antigravity workflows
(`.agents/workflows/opsx-*.md`) are left behind, so `/opsx-propose` and friends still show
up and point at a CLI the project no longer uses. And only the Claude adapter is installed,
so a project with an `AGENTS.md`, `.cursor/` or `.codex/` comes out of the migration with
nothing telling those agents about specjutsu.

## What Changes

- `--from openspec` also removes OpenSpec's Antigravity workflows: `<agent>/workflows/opsx-<name>.md`
  files whose frontmatter holds only a `description` and whose body runs the `openspec` CLI.
  Other workflows, including an `opsx-*` file that never runs `openspec`, are kept.
- Every migration installs the Claude adapter plus every adapter `sj init --agents auto` would
  install for the agent files the project already has (`AGENTS.md`, `.codex/`, `.cursor/`,
  `.clinerules/`). The preview lists their files.
- Agent detection moves from `sj-cli` to `sj-adapters` so `sj init`, `sj update` and
  `sj migrate` share one list.

## Capabilities

### New Capabilities

### Modified Capabilities

- `migration`: removes OpenSpec's Antigravity workflows, and installs adapters for detected agents.

## Impact

- `crates/sj-core/src/migrate/mod.rs`, a new `crates/sj-adapters/src/agents.rs`,
  `crates/sj-cli/src/main.rs` (uses the shared detection), `crates/sj-cli/tests/migrate_cmd.rs`.
- README's migration section.
