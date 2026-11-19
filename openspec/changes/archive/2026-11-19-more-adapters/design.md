## Context

The Claude Code adapter (ADAPT-001..010) installs a marker block, skills and hooks, and `sj hook claude pre-edit` decides from Claude's hook JSON. Its decision logic is in `sj-core::hook`, and the settings editing is in `sj-adapters` (`jsonedit`, `block`). The other agents differ only in where their instructions and hooks live and in their hook JSON:

| Agent | Instructions | Pre-tool hook | Hook answer |
| --- | --- | --- | --- |
| Codex | `AGENTS.md` | `[[hooks.PreToolUse]]` in `.codex/config.toml`, matcher `^(apply_patch\|Edit\|Write)$` | deny: `hookSpecificOutput.permissionDecision = "deny"`; allow: no output, exit 0 |
| Cursor | `AGENTS.md` (also `.cursor/rules/`) | `preToolUse` in `.cursor/hooks.json`, matcher `Write\|Delete` | `{"permission": "deny" \| "allow", "user_message", "agent_message"}` |
| Cline | `.clinerules/`, `AGENTS.md` | none here: TypeScript plugins in `.cline/plugins/`, CLI and SDK only | — |
| Amp, Roo Code, others | `AGENTS.md` | none supported here | — |

## Goals / Non-Goals

**Goals:**
- Every agent a team uses gets the same loop and, where its hooks allow, the same red-before-implementation rule as Claude Code.
- One shared instruction file where agents share one.
- Users can see how strongly each agent is held (ADAPT-020), so nobody believes a hook is enforcing what it isn't.

**Non-Goals:**
- Skills for agents other than Claude Code. Formats differ and change fast; the AGENTS.md block names `sj` commands instead, which every agent can run.
- Amp and Roo Code hooks. They get `agents-md` now; `sj hook check-edit` is the hook point when they add one.
- Windsurf, Copilot, Antigravity and Gemini CLI. They follow the same pattern in later changes.
- Session-start bootstrap hooks for the new agents. Their AGENTS.md block tells the agent to run `sj status` first.

## Decisions

### 1. Split the decision from the protocol
`sj-core::hook::decide(path, config, ledger, loop) -> Allow | Block(reason) | Warn(reason)` takes a path. Each agent has a small `Protocol` in `sj-adapters`, which parses its input into a path and renders the decision in its format. Claude's existing hook becomes one protocol, and its tests stay as they are. `sj hook check-edit` is the identity protocol.

### 2. AGENTS.md is the shared instruction file
Codex, Cursor, Amp and Roo Code all read `AGENTS.md`, so one block serves them all (ADAPT-011). Rejected: a rules file per agent (`.cursor/rules/specjutsu.mdc`, `.roo/rules/…`). It means more files to keep in sync, and a project's AGENTS.md is already where teams put shared rules.

### 3. Config files are edited in place
`.cursor/hooks.json` uses the existing span-keeping `jsonedit`, shared with Claude's `settings.json`, and our entry is recognised by its command starting with `sj hook cursor `. `.codex/config.toml` gets our hook group appended as text between `# specjutsu:begin` and `# specjutsu:end` comments. Install checks with `toml` that the result still parses and writes nothing if it doesn't, and removal cuts exactly the block and the line break before it, so the file comes back byte-identical (ADAPT-013). Rejected: `toml_edit`. It keeps comments, but removing an array-of-tables entry doesn't promise the original bytes back, and it would be a new dependency. A file a remover leaves empty (or, for Cursor, holding only `version`) is deleted, with its folder if that leaves it empty.

### 4. Cline gets instructions only
Cline's hooks are now TypeScript plugins (`.cline/plugins/*.ts|js`, a `beforeTool` hook returning `{skip, reason}`) that run only in the Cline CLI and SDK, not in the VS Code or JetBrains extensions most Cline users run. A plugin would enforce for a minority and still need a JS runtime bridge to `sj`. So `cline` installs the AGENTS.md block and doctor reports `cline: instructions` (ADAPT-022). Rejected: a generated `.cline/plugins/specjutsu.js` that spawns `sj hook cline pre-edit`; it's a later change once the plugin API reaches the extensions. Since the adapter writes only the shared block, `sj doctor` counts Cline as installed when the AGENTS.md block exists and the repository has `.clinerules`.

### 6. Paths come from each agent's own input
Codex sends `apply_patch` edits as patch text in `tool_input.command`; one patch can touch several files. The Codex protocol collects every `*** Add File:`, `*** Update File:`, `*** Delete File:` and `*** Move to:` path and denies if any one is blocked, naming it (ADAPT-021). Rejected: deciding on the first path only, which lets a source edit ride along with a test edit. Cursor's docs don't name the path key for `Write`/`Delete`, so the protocol takes the first string among `file_path`, `path` and `target_file` (ADAPT-016).

### 5. Fail open everywhere
As for Claude (ADAPT-009), any parse or internal error allows the edit. Cursor treats output that isn't valid JSON as a deny, so its protocol always prints `{"permission":"allow"}` or a deny object, never nothing (ADAPT-018). A broken hook that blocks work gets uninstalled, and the CI gate catches what slipped through.

## Risks / Trade-offs

- [Agents' hook formats change between releases] → Each protocol is small and has fixture tests built from the agent's documented examples. `sj doctor` points at the installed `sj` version, so `sj update` refreshes the hook entries.
- [Codex runs project hooks only in trusted projects, and only after the user trusts each hook in `/hooks`] → The README tells Codex users to trust the hook once; until they do, the CI gate is the backstop. Codex's own docs call tool hooks "a useful guardrail, not a complete enforcement boundary".
- [The Codex patch format or Cursor's path key differ from what the docs imply] → Unknown shapes fail open with a warning, and the fixture tests are the place to correct them once confirmed on a real build.
- [Cursor bugs where deny didn't block] → Documented. The gate in CI is the backstop, and ADAPT-020 makes the level visible.
