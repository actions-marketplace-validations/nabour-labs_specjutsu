## Context

`more-adapters` splits the pre-edit decision from each agent's hook format (`decide(path, …)` plus a `Protocol` per agent), adds the shared `AGENTS.md` block, `auto` detection and enforcement levels. It lists Windsurf, Copilot, Antigravity and Gemini CLI as later work. This change is that work, plus shared skills, which OpenSpec 2.0 moved to `.agents/skills/` for every agent.

What each agent offers as of Nov 2026:

| Agent | Instructions | Pre-tool hook | Hook answer |
| --- | --- | --- | --- |
| Gemini CLI | `GEMINI.md`; others via `context.fileName` in settings | `hooks.BeforeTool` in `.gemini/settings.json`, matcher on tool name (`write_file\|replace`) | JSON `{"decision":"deny","reason":…}` on stdout, exit 0 (exit 2 is a hard stop) |
| GitHub Copilot (CLI, cloud agent) | `.github/copilot-instructions.md`, `AGENTS.md` | `PreToolUse` (PascalCase key: Claude-shaped `tool_name`/`tool_input`, Claude tool names) in `.github/hooks/*.json`, `"version": 1`, `bash` entries; the cloud agent reads it from the default branch only | `{"permissionDecision":"deny","permissionDecisionReason":…}` on stdout, exit 0; any non-zero exit also denies, so the hook never exits non-zero |
| Windsurf (Cascade) | `.windsurf/rules/`, `AGENTS.md` | `pre_write_code` in `.windsurf/hooks.json`, input `tool_info.file_path`; system, user and workspace levels merged | exit 2 blocks; stderr shown |
| Kiro | `.kiro/steering/`, `AGENTS.md` | `PreToolUse` in `.kiro/hooks/*.json` (Kiro 3.0 moved hooks out of agent configs), matcher category `write`; input `tool_name`, `tool_input` | any non-zero exit blocks (2 used); stderr returned to the agent |
| Antigravity | `AGENTS.md`, `GEMINI.md`, `.agents/rules/` (legacy `.agent/rules/`) | `PreToolUse` in `.agents/hooks.json` (workspace) or `~/.gemini/config/hooks.json`; the IDE runs workspace hooks; input `toolCall.name`, `toolCall.args`; write tools `write_to_file`, `replace_file_content`, `multi_replace_file_content` | `{"decision":"deny","reason":…}` on stdout |

Sources: GitHub's Copilot CLI hooks reference, kiro.dev hooks docs, antigravity.google hooks docs. Windsurf's hooks page now redirects to Devin docs, so its input shape comes from the older Windsurf docs.

Path keys the docs don't name are read from a short list of likely keys, as Cursor's are in `more-adapters` (ADAPT-016): Copilot `file_path`, `path`; Kiro `path`, `file_path`; Antigravity `TargetFile`, `file_path`, `path`. `more-adapters-3` task 2.1 checks each against a real install before release.

## Goals / Non-Goals

**Goals:**
- Every agent OpenSpec users commonly run gets the loop, and a hook wherever the agent can block a write.
- One copy of the skills for every agent that reads `.agents/skills/`.
- A per-user install for people who want the skills in every project without touching each repository.

**Non-Goals:**
- The other agents in OpenSpec's list (Trae, Crush, Devin Desktop, Kilo, …). They read `AGENTS.md` and can call `sj hook check-edit`.
- Copilot in VS Code chat. It has its own preview hooks; this change targets the Copilot CLI and cloud agent, which read `.github/hooks/`.
- Global hooks or global instruction blocks.

## Decisions

### 1. One protocol per answer style, not per agent
Windsurf and Kiro both answer with exit code 2 and stderr, which is what `sj hook check-edit` already does. They share one exit-code protocol and differ only in how the path is read from stdin. Gemini, Copilot and Antigravity each get a JSON protocol. Rejected: a separate protocol module per agent with duplicated exit-code handling.

### 2. Gemini reads AGENTS.md through its own setting
Gemini CLI reads `GEMINI.md` unless `context.fileName` names other files. The adapter adds `AGENTS.md` to that list instead of writing a second block into `GEMINI.md`. Rejected: a `GEMINI.md` block, which would be one more copy to keep in step with `AGENTS.md`.

### 3. Gemini denies with JSON, not exit code 2
The Gemini docs keep exit code 2 for hard stops and prefer `{"decision":"deny"}` for a deliberate block, which lets the agent retry with a test. Rejected: exit 2, which ends the turn.

### 4. Antigravity waits for confirmed IDE hooks
Its hooks page documents `PreToolUse`, but whether the IDE runs workspace hooks is an open question in Google's own forum. Installing a hook that may silently not run would make `sj doctor` report `hook` falsely, the exact thing ADAPT-020 exists to prevent. So it is `instructions` level now.

### 5. Shared skills are files, not links
OpenSpec 2.0 links `.claude/skills/` to `.agents/skills/`. Symbolic links need extra rights on Windows and are often checked in as plain text files, so specjutsu writes two identical copies and `sj update` keeps them in step (ADAPT-032). Rejected: symbolic links, which break the principle that `sj` works on Windows, macOS and Linux.

### 6. Global install is skills only
Hooks and instruction blocks are a repository's rules and must be in the repository for CI and teammates to see them. `--global` only saves people from committing skill files (ADAPT-034). Rejected: global hooks, which would enforce a loop on repositories that never adopted specjutsu.

## Risks / Trade-offs

- [Hook formats move fast; Kiro and Copilot changed theirs in 2026] → each protocol is small, fail-open (ADAPT-031), and `sj doctor` shows `hook missing` when an entry stops matching.
- [Copilot's cloud agent supports only some hook events] → `sj doctor` reports `copilot: hook` for the repository; the CI gate remains the backstop.
- [Two skill copies can drift if edited by hand] → `sj update` reports the hand-edited copy and keeps it, as for `.claude/skills/` today.

- [Windsurf's hook docs moved to Devin Desktop, which uses `.devin/hooks.v1.json`] → `windsurf` keeps `.windsurf/hooks.json` for current Windsurf installs. A `devin` adapter is a later change.
- [Antigravity hooks reported not firing in IDE 1.107.0 on Windows] → doctor still says `hook` because the hook is installed. The CI gate is the backstop, and the manual notes the report.
- [Guessed path keys are wrong] → an unknown key means no path, which allows the edit with a warning (fail-open). `more-adapters-3` task 2.1 confirms the keys against a real install.
