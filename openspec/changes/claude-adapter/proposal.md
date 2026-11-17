## Why

Superpowers enforces TDD with prompts. OpenSpec and Spec Kit install their own skills and slash commands. To replace all three in Claude Code, specjutsu must install its own instructions, skills and hooks. The hooks must call back into the binary, so enforcement is deterministic and matches CI. It must also coexist with project hooks you already rely on: `ellesmere.ai` runs `require-worktree.mjs` before every edit, and its settings carry its own deny rules.

## What Changes

- `sj init --agents claude` (and `sj update`) writes: a CLAUDE.md block between `<!-- specjutsu:begin -->` and `<!-- specjutsu:end -->` markers, the skills from `workflow-skills` under `.claude/skills/sj-*/`, and hooks in `.claude/settings.json`.
- Hooks: `SessionStart` runs `sj hook claude session-start`, which prints the bootstrap (status in a few lines). `PreToolUse` on `Edit|Write|MultiEdit|NotebookEdit` runs `sj hook claude pre-edit`.
- `pre-edit` decides by mode. In `guided` and `strict` it blocks an edit to an implementation file while the active task's scenarios have no failing or build-failed ledger record on the current tree. Test files, specs and docs are always allowed. In `advisory` it warns.
- Generated files carry no timestamps, so prompt caches stay valid. `sj update` is idempotent and keeps every setting, hook and permission it did not write.

## Capabilities

### New Capabilities
- `adapters`: generating, updating and removing agent-tool integration files, and the `sj hook` entrypoint. The first adapter is Claude Code. Cursor, Codex, Copilot and Antigravity add requirements here later.

### Modified Capabilities
None.

## Impact

- `sj-adapters` gets its first real code: a JSON merge for `.claude/settings.json` that preserves key order, and a marker-block writer for CLAUDE.md.
- `adapters/claude/` holds the templates. The skill bodies come from `skills/`.
- Depends on `agent-loop` (bootstrap content, active task) and `test-ledger` (red evidence for `pre-edit`).
