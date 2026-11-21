## Context

The research for these agents is in `more-adapters-2`'s design.md (`changes/archive/2026-11-21-more-adapters-2/`): the table of each agent's instructions file, pre-tool hook and hook answer as of Nov 2026, and the sources. This change carries the Copilot, Windsurf and Kiro rows of that table.

## Goals / Non-Goals

**Goals:**
- A pre-edit hook in Copilot (CLI and cloud agent), Windsurf and Kiro.

**Non-Goals:**
- Copilot in VS Code chat, which has its own preview hooks.
- A `devin` adapter for Devin Desktop's `.devin/hooks.v1.json`.

## Decisions

### 1. One protocol per answer style, not per agent
Windsurf and Kiro both answer with exit code 2 and stderr, which is what `sj hook check-edit` already does. They share one exit-code protocol and differ only in how the path is read from stdin. Copilot gets a JSON protocol. Rejected: a separate protocol module per agent with duplicated exit-code handling.

### 2. Copilot never exits non-zero
Copilot denies the tool call on any non-zero exit, so an internal error or unreadable input must exit 0 with nothing on stdout, or every edit would be blocked (ADAPT-026).

## Risks / Trade-offs

- [Hook formats move fast; Kiro and Copilot changed theirs in 2026] → each protocol is small, fail-open (ADAPT-031), and `sj doctor` shows `hook missing` when an entry stops matching.
- [Copilot's cloud agent supports only some hook events] → `sj doctor` reports `copilot: hook` for the repository; the CI gate remains the backstop.
- [Windsurf's hook docs moved to Devin Desktop] → `windsurf` keeps `.windsurf/hooks.json` for current Windsurf installs.
- [Guessed path keys are wrong] → an unknown key means no path, which allows the edit with a warning. Task 2.1 confirms the keys against a real install.

## Vendor check (task 2.1, Nov 21 2026)

Checked against each vendor's published docs, not against running installs.

| Agent | Path key | Hook file | Answer | Result |
| --- | --- | --- | --- | --- |
| Copilot | Not documented for the edit tools. PascalCase `PreToolUse` sends snake_case `tool_name` and `tool_input` (parsed from JSON) with Claude-style names (`create` → `Write`, `edit` → `Edit`). | `.github/hooks/*.json`, `"version": 1`, `hooks` keyed by event; entry `type`, `bash`, `matcher` (full-string regex on the tool name). | stdout `permissionDecision` / `permissionDecisionReason`; non-zero exit denies. | Matches. `file_path` then `path` stays a guess. The docs don't say the cloud agent reads hooks only from the default branch; the install notice is kept as a caution. |
| Windsurf | `tool_info.file_path` in `pre_write_code` (with `agent_action_name`). | Workspace `.windsurf/hooks.json`, `hooks` keyed by event, entry `command`, `show_output`. Devin Desktop (Windsurf's successor) prefers `.devin/hooks.json` and reads `.windsurf/hooks.json` only when that file is absent or empty. | Exit 2 blocks. | Matches. Windsurf's own page now redirects to Devin's docs, so this rests on the older Windsurf page and third-party copies of it. |
| Kiro | `tool_input.path` (shown for `read_file`; the write tool's key isn't documented). | `.kiro/hooks/<id>.json`: `{"version": "v1", "hooks": [{"name", "trigger": "PreToolUse", "matcher", "action": {"type": "command", "command"}}]}`. `write` is a documented category, "all built-in file write tools". | V3: any non-zero exit blocks `PreToolUse`, stderr goes to the agent; 2.x: exit 2 blocks. | **Fixed:** the file used `hooks` keyed by event; it now uses Kiro's schema (ADAPT-029). The hook never exits non-zero on bad input, which matters because V3 fails closed. |
| Antigravity | `toolCall.args.TargetFile` for `write_to_file`, `replace_file_content` and `multi_replace_file_content`. | `.agents/hooks.json`, top-level keys are hook names, each holding events: `{"<name>": {"PreToolUse": [{"matcher", "hooks": [{"type", "command"}]}]}}`. | stdout `decision` (`allow`, `deny`, `ask`, ...) and `reason`; exit codes not documented. | **Fixed:** the entry sat under a top-level `hooks` key, which Antigravity doesn't read; it now goes under a `specjutsu` hook (ADAPT-030). Install and removal also take out an entry in the old shape. |

Sources: docs.github.com/en/copilot/reference/hooks-reference, docs.github.com/en/copilot/concepts/agents/coding-agent/about-hooks, docs.devin.ai/desktop/cascade/hooks, kiro.dev/docs/hooks/ and /docs/cli/v3/hooks-migration/, antigravity.google/docs/hooks/.
