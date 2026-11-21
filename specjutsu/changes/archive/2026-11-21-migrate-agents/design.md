## Context

`sj migrate` removes OpenSpec files only when their own metadata proves OpenSpec wrote them,
then installs the Claude adapter. v0.4.0 added Codex, Cursor, Cline and `AGENTS.md` adapters
with `sj init --agents auto` detection, but migrate never learned about them. OpenSpec's
Antigravity workflows carry no generator metadata: just a `description`.

## Goals / Non-Goals

**Goals:**
- No OpenSpec command stays reachable from any agent after migration.
- Every agent the project already uses is told about specjutsu.

**Non-Goals:**
- An Antigravity adapter or skills in `.agents/skills/`: that is `more-adapters-2` (v0.6).
- Rewriting hand-written `AGENTS.md` prose that mentions OpenSpec or Superpowers.

## Decisions

- **Antigravity workflow test.** Name `opsx-<name>.md` under `<agent>/workflows/`, frontmatter
  with exactly one key, `description`, and a body that runs the `openspec` CLI (a line or code
  span containing `openspec ` followed by a subcommand). Rejected: name alone, which could
  remove a project's own `opsx-*` file; a fixed list of OpenSpec's descriptions, which breaks
  whenever OpenSpec rewords them.
- **Adapters = Claude + auto.** Migrate resolves `["claude", "auto"]` with the same function
  `sj init` uses. Claude stays unconditional so MIG-007 is unchanged. Rejected: a new
  `--agents` flag on migrate; detection covers the real cases and the flag can come later.
- **Re-plan at apply.** The adapter plan is recomputed after the other writes, as the Claude
  install already was, because the Superpowers step edits `.claude/settings.json` first.
- **Detection moves to `sj-adapters`.** One `agents` module with `detect`, `resolve` and
  `plan`; `sj-cli` calls it. Rejected: duplicating the list in `sj-core`.

## Risks / Trade-offs

- [A project's own workflow matches all three conditions] → It would have to be named
  `opsx-*`, hold only a description and run `openspec`; the preview lists it before `--write`.
