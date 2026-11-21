## Why

After `more-adapters`, specjutsu reaches Claude Code, Codex, Cursor, Cline and every agent that reads `AGENTS.md`. OpenSpec reaches about 30 agents and, since 2.0, gives all of them the same skills from one shared `.agents/skills/` folder. Teams on GitHub Copilot, Gemini CLI, Windsurf, Kiro or Antigravity still get no pre-edit hook from specjutsu, and every agent except Claude Code gets no skills. Most of these agents now have a pre-tool hook that can block a file write, so the red-before-implementation rule can run inside them too.

**Depends on `more-adapters`.** This change reuses its `decide(path, …)` split, per-agent protocols, the `agents-md` block, `auto` detection and the enforcement levels in `sj doctor`. It is implemented after `more-adapters` is archived.

## What Changes

- `sj init --agents` accepts `gemini` and `antigravity`. Each implies `agents-md`, and `auto` detects them.
- `gemini` adds a `BeforeTool` hook in `.gemini/settings.json` that calls `sj hook gemini pre-edit`, and adds `AGENTS.md` to `context.fileName`.
- `antigravity` adds a `PreToolUse` hook in `.agents/hooks.json` that blocks source edits before red, as the other hook agents do.
- The `sj-*` skills are also written to the shared `.agents/skills/` folder when any installed agent reads it (Codex, Gemini CLI, Antigravity). Claude Code keeps `.claude/skills/`.
- `sj init --global` installs the skills once into the user's home folders for every project. It never writes hooks, because hooks belong to a repository.
- `sj doctor` reports the new agents' enforcement levels and where the skills are installed.
- GitHub Copilot, Windsurf and Kiro follow in `more-adapters-3`, which also holds the check against real installs and the docs.

## Capabilities

### New Capabilities
None.

### Modified Capabilities
- `adapters`: Gemini CLI and Antigravity, their hook protocols, shared skills and global install. The unknown-agent scenario (ADAPT-012) no longer pins the full list of names. The Claude Code, Codex, Cursor and Cline requirements are otherwise unchanged.

## Impact

- `sj-adapters`: `antigravity`, `gemini`, `shared_skills` and `global` modules; Claude Code's skill planning is shared by `.claude/skills/` and `.agents/skills/`. Config editors for `.gemini/settings.json` and `.agents/hooks.json` reuse `jsonedit`.
- `sj-cli`: the new agent names, `sj hook gemini|antigravity pre-edit`, and `--global` on `sj init`, `sj update` and `sj init --remove-agents`.

## Clarifications

- Does Antigravity's IDE run workspace hooks? → Yes, from `.agents/hooks.json`. It gets a hook now, and doctor reports `antigravity: hook`.
