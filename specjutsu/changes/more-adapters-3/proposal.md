## Why

`more-adapters-2` gives Gemini CLI and Antigravity a pre-edit hook, and shares the `sj-*` skills through `.agents/skills/`. Teams on GitHub Copilot, Windsurf or Kiro still get no pre-edit hook from specjutsu. Each of these agents now has a pre-tool hook that can block a file write, so the red-before-implementation rule can run inside them too.

**Depends on `more-adapters-2`.** This change reuses its agent list, `auto` detection, JSON hook editing and shared skills. It was split out of `more-adapters-2` so that change could land with only the work that is built. It is implemented after `more-adapters-2` is archived.

## What Changes

- `sj init --agents` accepts `copilot`, `windsurf` and `kiro`. Each implies `agents-md`, and `auto` detects them.
- `copilot` writes `.github/hooks/specjutsu.json`, `windsurf` adds a `pre_write_code` entry to `.windsurf/hooks.json`, and `kiro` writes `.kiro/hooks/specjutsu.json`. Each calls `sj hook <agent> pre-edit`, with the same decision as the Claude hook.
- `sj doctor` reports the new agents' enforcement levels.
- The README and manual "Agent setup" table list every agent from `more-adapters-2` and this change.

## Capabilities

### New Capabilities
None.

### Modified Capabilities
- `adapters`: three more agents and their hook protocols. Existing requirements are unchanged.

## Impact

- `sj-adapters`: protocols for Copilot (JSON on stdout, never a non-zero exit), Windsurf and Kiro (exit code 2); config editors for `.github/hooks/specjutsu.json`, `.windsurf/hooks.json` and `.kiro/hooks/specjutsu.json`.
- `sj-cli`: the new agent names and `sj hook copilot|windsurf|kiro pre-edit`.
- README and manual "Agent setup" support table; CHANGELOG.

## Clarifications

- Copilot's deny output and path field? → Register under the PascalCase `PreToolUse` key, which gives Claude-shaped input. Block with `{"permissionDecision":"deny","permissionDecisionReason":…}` and exit 0, and never exit non-zero, because Copilot denies on any non-zero exit. The path comes from `tool_input.file_path` or `path`, or from the paths in an `apply_patch` patch.
- Windsurf's `pre_write_code` path field? → `tool_info.file_path`.
- Kiro's file-write tool names for the matcher? → Match Kiro's `write` category, so the tool IDs aren't needed. The path comes from `tool_input.path` or `file_path`.
