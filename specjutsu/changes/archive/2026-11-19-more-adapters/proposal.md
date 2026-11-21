## Why

specjutsu only installs into Claude Code. Teams that want a quality gate for AI-generated code rarely use one agent: the same repository sees Claude Code, Codex, Cursor, Cline, Roo Code and Amp, often in one week. A rule that only one of them follows isn't a team rule. The CI gate covers every agent, but the cheapest place to stop nonsense is before the edit, inside the agent's own loop. Several agents now have pre-tool hooks: Cursor (`.cursor/hooks.json`) and Codex (`[hooks]` in `.codex/config.toml`); Cline's hooks are TypeScript plugins for its CLI only. Most of them, plus Amp and Roo Code, read `AGENTS.md`.

## What Changes

- `sj init --agents` accepts a comma-separated list: `claude`, `codex`, `cursor`, `cline`, `agents-md`, and `auto`, which detects the agents already configured in the repository.
- `agents-md` writes the specjutsu block into `AGENTS.md`, with the same markers and rules as the CLAUDE.md block. It is the instruction path for every agent that reads AGENTS.md (Codex, Cursor, Amp, Roo Code and others), and it's written once however many of them are installed.
- `codex` and `cursor` add pre-edit hooks that call `sj hook <agent> pre-edit`. It makes the same decision as the Claude hook (ADAPT-006..009) and translates the agent's own hook input and output. `cline` installs `agents-md` only.
- `sj hook check-edit <path>` exposes the decision with no agent-specific JSON, for any agent or editor with a hook system we don't support yet.
- `sj doctor` reports each installed agent's enforcement level: `hook` (edits are blocked before red) or `instructions` (the agent is told the loop; the CI gate is the backstop).

## Capabilities

### New Capabilities
None.

### Modified Capabilities
- `adapters`: adds requirements for the new agents, the shared AGENTS.md block, agent detection, the generic edit check and enforcement reporting. The Claude Code requirements are unchanged.

## Impact

- `sj-adapters`: a marked hook block appended to `.codex/config.toml` and checked with `toml`, reuse of `jsonedit` for `.cursor/hooks.json`.
- `sj-core::hook`: the pre-edit decision takes a path, not Claude's JSON. Per-agent input parsing and output shaping move to `sj-adapters`.
- `adapters/<agent>/` templates; README and manual "Agent setup" section with a support table.
- No change to existing Claude Code installs: `sj update` leaves them byte-identical.

## Clarifications

- Codex — do `PreToolUse` hooks fire for `apply_patch` edits? → Yes (Codex hooks docs): matcher `apply_patch|Edit|Write`, paths parsed from the patch in `tool_input.command`, deny if any path is blocked (ADAPT-021).
- Cursor — deny format and edit tool names? → `{"permission":"deny","user_message","agent_message"}`; tools `Write` and `Delete`. The path key is undocumented, so try `file_path`, `path`, `target_file`; always print valid JSON, since Cursor blocks on anything else (ADAPT-016, ADAPT-018).
- Cline — the proposed `.clinerules/hooks/PreToolUse` script doesn't exist in current Cline; hooks are CLI-only TypeScript plugins → `cline` is instructions-only in this change (ADAPT-022); ADAPT-017 is dropped and not reused.
