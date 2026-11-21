## Why

After `more-adapters`, specjutsu reaches Claude Code, Codex, Cursor, Cline and every agent that reads `AGENTS.md`. OpenSpec reaches about 30 agents and, since 2.0, gives all of them the same skills from one shared `.agents/skills/` folder. Teams on GitHub Copilot, Gemini CLI, Windsurf, Kiro or Antigravity still get no pre-edit hook from specjutsu, and every agent except Claude Code gets no skills. Most of these agents now have a pre-tool hook that can block a file write, so the red-before-implementation rule can run inside them too.

**Depends on `more-adapters`.** This change reuses its `decide(path, …)` split, per-agent protocols, the `agents-md` block, `auto` detection and the enforcement levels in `sj doctor`. It is implemented after `more-adapters` is archived.

## What Changes

- `sj init --agents` accepts `copilot`, `gemini`, `windsurf`, `kiro` and `antigravity`. Each implies `agents-md`, and `auto` detects them.
- `gemini`, `copilot`, `windsurf` and `kiro` add a pre-edit hook that calls `sj hook <agent> pre-edit`, with the same decision as the Claude hook.
- `antigravity` adds a `PreToolUse` hook in `.agents/hooks.json` that blocks source edits before red, as the other hook agents do.
- The `sj-*` skills are also written to the shared `.agents/skills/` folder when any installed agent reads it. Claude Code keeps `.claude/skills/`.
- `sj init --global` installs the skills once into the user's home folders for every project. It never writes hooks, because hooks belong to a repository.
- `sj doctor` reports the new agents' enforcement levels and where the skills are installed.

## Capabilities

### New Capabilities
None.

### Modified Capabilities
- `adapters`: new agents, their hook protocols, shared skills and global install. The Claude Code, Codex, Cursor and Cline requirements are unchanged.

## Impact

- `sj-adapters`: protocols for Gemini CLI (JSON decision), Copilot (JSON on stdin), Windsurf and Kiro (exit code 2); config editors for `.gemini/settings.json` (reuse `jsonedit`), `.github/hooks/specjutsu.json`, `.windsurf/hooks.json` and `.kiro/hooks/specjutsu.json`.
- `adapters/<agent>/` templates; skill templates rendered into `.agents/skills/`.
- `sj-cli`: `--global` flag on `sj init`, `sj update` and `sj init --remove-agents`.
- README and manual "Agent setup" support table.

## Clarifications

- Copilot's deny output and path field? → Register under the PascalCase `PreToolUse` key, which gives Claude-shaped input. Block with `{"permissionDecision":"deny","permissionDecisionReason":…}` and exit 0, and never exit non-zero, because Copilot denies on any non-zero exit. The path comes from `tool_input.file_path` or `path`, or from the paths in an `apply_patch` patch.
- Windsurf's `pre_write_code` path field? → `tool_info.file_path`.
- Kiro's file-write tool names for the matcher? → Match Kiro's `write` category, so the tool IDs aren't needed. The path comes from `tool_input.path` or `file_path`.
- Does Antigravity's IDE run workspace hooks? → Yes, from `.agents/hooks.json`. It gets a hook now, and doctor reports `antigravity: hook`.
